use std::{env, fs};

use ap2024_rustinpeace_nosounddrone::NoSoundDroneRIP;
use common::slc_commands::TextMediaResponse;
use dr_ones::Drone as DrDrone;
use drone_bettercalldrone::BetterCallDrone;
use getdroned::GetDroned;
use rolling_drone::RollingDrone;
use rust_do_it::RustDoIt;
use rust_roveri::RustRoveri;
use rustafarian_drone::RustafarianDrone;
use rusteze_drone::RustezeDrone;
use rusty_drones::RustyDrone;

use super::{generic_full_file_request, instanciate_testing_topology};

#[test]
fn test_drdrone() {
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
        instanciate_testing_topology::<DrDrone>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}

#[test]
fn test_bettercalldrone() {
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
        instanciate_testing_topology::<BetterCallDrone>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}

#[test]
fn test_getdroned() {
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
        instanciate_testing_topology::<GetDroned>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}

#[test]
fn test_rustinpeace() {
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
        instanciate_testing_topology::<NoSoundDroneRIP>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}

#[test]
fn test_rollingdrone() {
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
        instanciate_testing_topology::<RollingDrone>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}

#[test]
fn test_rustdoit() {
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
        instanciate_testing_topology::<RustDoIt>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}

#[test]
fn test_rustroveri() {

    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
        instanciate_testing_topology::<RustRoveri>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}

#[test]
fn test_rustafarian() {
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
        instanciate_testing_topology::<RustafarianDrone>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}

#[test]
fn test_rusteze() {
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
        instanciate_testing_topology::<RustezeDrone>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}

#[test]
fn test_rustydrone() {
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
        instanciate_testing_topology::<RustyDrone>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}
