#![feature(test)]
extern crate test;
extern crate gutenberg;
extern crate tempdir;

use std::env;

use tempdir::TempDir;
use gutenberg::{Site, populate_previous_and_next_pages};

// TODO: add bench with ~1000 pages for all cases

#[bench]
fn bench_loading_test_site(b: &mut test::Bencher) {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();


    b.iter(|| site.load().unwrap());
}


#[bench]
fn bench_building_test_site(b: &mut test::Bencher) {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();
    let tmp_dir = TempDir::new("example").expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);


    b.iter(|| site.build().unwrap());
}

#[bench]
fn bench_populate_previous_and_next_pages(b: &mut test::Bencher) {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();
    let pages = site.pages.values().cloned().collect::<Vec<_>>();

    b.iter(|| populate_previous_and_next_pages(pages.as_slice()));
}
