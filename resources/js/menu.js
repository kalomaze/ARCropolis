
var AButtonHeld = [false, false, false, false];

window.onload = function () {
    var buttons = document.querySelectorAll('button');

    [].forEach.call(buttons, function (btn) {
        btn.addEventListener("focus", () => {
            btn.classList.add("is-focused");
        });

        btn.addEventListener("focusout", () => {
            btn.classList.remove("is-focused");
        });
    });

    // Listen to the keydown event and prevent the default
    window.addEventListener('keydown', function (e) {
        e.preventDefault();
    });

    // Listen to the gamepadconnected event
    window.addEventListener("gamepadconnected", function (e) {
        if ($(".is-focused").length <= 0) {
            $("#list").find("button").get(0).focus();
        }
        // Once a gamepad has connected, start an interval function that will run every 100ms to check for input
        setInterval(function () {
            var gpl = navigator.getGamepads();
            if (gpl.length > 0) {
                for (var i = 0; i < gpl.length; i++) {
                    checkGamepad(i, gpl[i]);
                }
            }
        }, 100);
    });
}

function checkGamepad(index, gamepad) {
    //#region UI Input Check

    var axisX = gamepad.axes[0];
    var axisY = gamepad.axes[1];

    // Check A button
    if (gamepad.buttons[1].pressed) {
        if (!AButtonHeld[index]) {
            AButtonHeld[index] = true;
            $(".is-focused").last().click();
        }
    } else {
        AButtonHeld[index] = false;
    }

    // Check if D-pad Left pressed or Left Stick X Axis less than -0.7
    if (gamepad.buttons[14].pressed || axisX < -0.7) {
        // Go up by 6 elements
        var slice_index = 6;
        var target = $(".is-focused").prevAll(":visible").slice(0, slice_index).last();
        while (target.length <= 0 && slice_index != 0) {
            slice_index -= 1;
            target = $(".is-focused").prevAll(":visible").slice(0, slice_index).last();
        }

        // If that doesn't exist, then dip
        if (target.length <= 0) {
            return;
        }
        scroll(target, $("#list").scrollTop() + target.position().top - 50);
    }
    // Check if D-pad Up pressed or Y-Axis
    else if (gamepad.buttons[12].pressed || axisY < -0.7) {
        // Get the mod above the current focused one
        var target = $(".is-focused").prev();

        while (target.length > 0 && target.is(':hidden')) {
            target = target.prev();
        }

        // If that doesn't exist, then dip
        if (target.length <= 0) {
            return;
        }

        scroll(target, $("#list").scrollTop() + target.position().top - 50);
    }
    // Check if D-pad Right pressed or X Axis > 0.7
    else if (gamepad.buttons[15].pressed || axisX > 0.7) {
        // Go up down 6 elements
        var slice_index = 6;
        var target = $(".is-focused").nextAll(":visible").slice(0, slice_index).last();

        while (target.length <= 0 && slice_index != 0) {
            slice_index -= 1;
            target = $(".is-focused").nextAll(":visible").slice(0, slice_index).last();
        }

        // If that doesn't exist, then dip
        if (target.length <= 0) {
            return;
        }

        scroll(target, ($("#list").scrollTop()) + (target.height() * 2));
    }
    // Check if D-pad Down pressed or Y Axis > 0.7
    else if (gamepad.buttons[13].pressed || axisY > 0.7) {
        // Get the next mod that will be focused on
        var target = $(".is-focused").next();

        while (target.length > 0 && target.is(':hidden')) {
            target = target.next();
        }

        console.log(target);
        // If there is none after that, then just return
        if (target.length <= 0) {
            return;
        }
        console.log(target);
        scroll(target, ($("#list").scrollTop()) + (target.height() * 2));
    };
    //#endregion
}

function scroll(target, offset) {
    // Check to see if mod is completely in view
    var fully = checkInView(target) == undefined ? false : true;

    // If so, then just focus and dip
    if (fully) {
        target.focus();
    } else {
        // Remove focus from currently focused one
        $(".is-focused").focusout();
        // Stop any animation going on in the container
        $("#list").stop();
        // Animate the mod container scrolling with a speed of 0 (fastest)
        $("#list").animate({
            scrollTop: offset
        }, 0);
        // Focus on the previous mod
        target.focus();
    }
}

// yonked from here https://stackoverflow.com/questions/16308037/detect-when-elements-within-a-scrollable-div-are-out-of-view
function checkInView(elem, partial) {
    var container = $("#list");
    var contHeight = container.height();
    var contTop = container.scrollTop();
    var contBottom = contTop + contHeight;

    var elemTop = $(elem).offset().top - container.offset().top;
    var elemBottom = elemTop + $(elem).height();

    var isTotal = (elemTop >= 0 && elemBottom <= contHeight);
    var isPart = ((elemTop < 0 && elemBottom > 0) || (elemTop > 0 && elemTop <= container.height())) && partial;

    return isTotal || isPart;
}

// Code to handle this session wasn't made to detect a closure by button
// window.nx.footer.unsetAssign( "B" );
