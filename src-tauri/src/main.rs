#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if synk_lib::boot::container::relaunch_outside_if_needed() {
        return;
    }
    synk_lib::run();
}
