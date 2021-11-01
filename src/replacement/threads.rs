use owo_colors::OwoColorize;

use smash_arc::{ArcLookup, Hash40};

use skyline::{
    hook,
    hooks::InlineCtx
};

use crate::{GLOBAL_FILESYSTEM, hashes, offsets, reg_w, reg_x, resource::{self, InflateFile, LoadInfo, LoadType}};

use super::FileInfoFlagsExt;

#[hook(offset = offsets::inflate(), inline)]
fn inflate_incoming(ctx: &InlineCtx) {
    let arc = resource::arc();
    let service = resource::res_service();

    let info_index = (service.processing_file_idx_start + reg_w!(ctx, 27)) as usize;
    let file_info = &arc.get_file_infos()[info_index];
    
    let file_path = &arc.get_file_paths()[file_info.file_path_index];
    let path_hash = file_path.path.hash40();

    info!(
        target: "no-mod-path",
        "[ResInflateThread::inflate_incoming | #{:#06X} | Type: {} | {:>3} / {:>3}] Incoming '{}'",
        usize::from(file_info.file_path_index).green(),
        reg_w!(ctx, 21).green(),
        reg_x!(ctx, 27).yellow(),
        service.processing_file_idx_count.yellow(),
        hashes::find(path_hash).bright_yellow()
    );

    let mut fs = GLOBAL_FILESYSTEM.write();

    let should_add = if let Some(path) = fs.hash(path_hash) {
        info!(
            "Added file '{}' to the queue.",
            path.display().yellow()
        );
        true
    } else {
        false
    };
    
    if should_add {
        fs.set_incoming(Some(path_hash));
    } else {
        fs.set_incoming(None);
    }
}

#[hook(offset = offsets::inflate_dir_file())]
fn inflate_dir_file(arg: u64, out_decomp_data: &mut InflateFile, comp_data: &InflateFile) -> u64 {
    trace!(
        target: "no-mod-path",
        "[ResInflateThread::inflate_dir_file] Incoming decompressed filesize: {:#x}",
        out_decomp_data.len()
    );

    let result = call_original!(arg, out_decomp_data, comp_data);

    let hash = crate::GLOBAL_FILESYSTEM.write().get_incoming();

    if let Some(hash) = hash {
        handle_file_replace(hash);
    }

    result
}

pub fn handle_file_replace(hash: Hash40) {
    let arc = resource::arc();
    let filesystem_info = resource::filesystem_info();

    let file_info = match arc.get_file_info_from_hash(hash) {
        Ok(info) => info,
        Err(_) => {
            error!("Failed to find file info for '{}' ({:#x}) when replacing.", hashes::find(hash), hash.0);
            return;
        }
    };

    let filepath_index = usize::from(file_info.file_path_index);
    let file_info_indice_index = usize::from(file_info.file_info_indice_index);

    let decompressed_size = arc.get_file_data(file_info, resource::res_service().language_idx).decomp_size;

    if filesystem_info.get_loaded_filepaths()[filepath_index].is_loaded == 0 {
        warn!(
            "When replacing file '{}' ({:#x}), the file is not marked as loaded. FilepathIdx: {:#x}, LoadedDataIdx: {:#x}",
            hashes::find(hash),
            hash.0,
            filepath_index,
            file_info_indice_index
        );
        return;
    }

    if filesystem_info.get_loaded_datas()[file_info_indice_index].data.is_null() {
        warn!(
            "When replacing file '{}' ({:#x}), the loaded data buffer is empty. FilepathIdx: {:#x}, LoadedDataIdx: {:#x}",
            hashes::find(hash),
            hash.0,
            filepath_index,
            file_info_indice_index
        );
        return;
    }

    let buffer = unsafe {
        std::slice::from_raw_parts_mut(
            filesystem_info.get_loaded_datas()[file_info_indice_index].data as *mut u8,
            decompressed_size as usize
        )
    };

    let fs = crate::GLOBAL_FILESYSTEM.read();

    if let Some(size) = fs.load_into(hash, buffer) {
        if arc.get_file_paths()[filepath_index].ext.hash40() == Hash40::from("nutexb") {
            if size < decompressed_size as usize {
                let (contents, footer) = buffer.split_at_mut((decompressed_size - 0xb0) as usize);
                footer.copy_from_slice(&contents[(size - 0xb0)..size]);
            }
        }
        info!("Replaced file '{}' ({:#x}).", hashes::find(hash), hash.0);
    }

}

// handles submitting files to be loaded manually
#[hook(offset = offsets::res_load_loop_start(), inline)]
fn res_loop_start(_: &InlineCtx) {
    res_loop_common();
}

#[hook(offset = offsets::res_load_loop_refresh(), inline)]
fn res_loop_refresh(_: &InlineCtx) {
    res_loop_common();
}

fn res_loop_common() {
    let arc = resource::arc();
    let service = resource::res_service_mut();
    let file_infos = arc.get_file_infos();
    let dir_infos = arc.get_dir_infos();

    let mut standalone_files = vec![Vec::new(); 5];

    for (list_idx, list) in service.res_lists.iter().enumerate() {
        for entry in list.iter() {
            match entry.ty {
                LoadType::Directory => {
                    for info in file_infos[dir_infos[entry.directory_index as usize].file_info_range()].iter() {
                        if info.flags.standalone_file() {
                            standalone_files[list_idx].push(info.file_path_index);
                        }
                    }
                },
                _ => {}
            }
        }
    }

    for (idx, vec) in standalone_files.into_iter().enumerate() {
        for path_idx in vec.into_iter() {
            service.res_lists[idx].insert(LoadInfo {
                ty: LoadType::File,
                filepath_index: path_idx.0,
                directory_index: 0xFF_FFFF,
                files_to_load: 0
            });
        }
    }
}

pub fn install() {
    skyline::install_hooks!(
        inflate_incoming,
        inflate_dir_file,

        res_loop_start,
        res_loop_refresh
    );
}