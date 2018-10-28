extern crate tempfile;
extern crate escargot;
extern crate reqwest;
extern crate unzip;

use tempfile::tempdir;
use std::fs::File;
//use std::path::Path;
use std::io::{self, Write};
use std::vec::Vec;
//use std::env;

#[macro_use]
extern crate duct;

//use duct::cmd;

#[test]
fn roundtrip() { //return -> std::io::Result<()> ?
//TODO: wrap test tasks in a function that returns Result so we can safely
//dispose of temp directories, and kill the server if test fails?

    //let mut file_vec = Vec::new();


  //generate some temporary files in a directory
  let dir= tempdir().unwrap();
  let archive_dir = tempdir().unwrap();
    {
        let dir_path = dir.path();//.to_owned();
        let archive_dir_path = archive_dir.path().to_str().unwrap();

        //let file = tempfile::tempfile_in(dir_path).unwrap();

        for i in 1..100 {
            // let mut file = tempfile::tempfile_in(dir_path).unwrap();

            //file_vec.push(file);
            let file_path= format!("{}/file{}", dir_path.to_str().unwrap(), i);
            let mut file = File::create(file_path).unwrap();
            writeln!(file, "here's some file contents");

        }

        //build and run this file with escargot:
        //assumption: defaults to building in ./target/debug
        escargot::CargoBuild::new()
            .current_target()
            .exec()
            .unwrap();

        let mut archive_path =archive_dir.path().to_str().unwrap().to_owned();
        archive_path.push_str("/docs.archive");

        //use duct to run the newly built binary,
        //and create docs.archive in directory for that binary
        let run_status = cmd!("./target/debug/static-filez", "build",
             dir.path().to_owned(), &archive_path )
             .read();//.unwrap();
        match run_status {
            Ok(o)=> {println!("{}", o)}
            Err(e)=> {
                panic!("Failed to archive files in {:?}: Error: {}",
                dir.path().to_owned().to_str(), e)
            }
        }


        //NOTE: must use .start() or test will hang indefinitely
        // since Result is only generated after server process
        // terminates for .run()
        let process_handle;
        let server_status = cmd!("./target/debug/static-filez",
             "serve", "-p", "3000", &archive_path)
             .start();
             match server_status{
                 Ok(o) => {println!("started server successfully."); process_handle = o;}
                 Err(e) => {panic!("unable to start archive hosting server: {}", e)}
             }

           // assert!(false, "killed test for debug");

        for i in 1 .. 100 {
           // let mut tmpfile = tempfile::tempfile_in(archive_dir.path())
                //.expect("failed to create temp file for zip file download");
            let mut tmpfile_path = format!("{}/zip{}", archive_dir_path, i);
            let mut tmpfile= File::create(tmpfile_path).expect("failed to create file to hold archived info");
            let mut unzipped_file_path= format!("{}/unZipped_file{}", &archive_dir_path, i).to_owned();
            //let x = unzipped_file_path.clone();
            //assert!(false, x);

            let url = String::from(format!("http://127.0.0.1:3000/file{}",i));


            let mut unzipped_tempfile=File::create(&unzipped_file_path).unwrap();

            let mut response = reqwest::get(&url).expect(&format!("failed to get file{} from server",i));
            io::copy(&mut response, &mut tmpfile).expect(
                &format!("failed to copy data for file{} from server response",i)
                );

            //while true{};
            //TODO: failing to extract server zipped file to another file:
            let extracter = unzip::Unzipper::new(tmpfile, &unzipped_file_path);
            let unzip_result = extracter.unzip().expect("failed to unzip file");
            let mut unzipped_contents = String::new();
            let res= std::fs::read_to_string(&unzipped_file_path);
            match res {
                Ok(o) => {unzipped_contents=o.clone()}
                Err(e) => {panic!("could not check unzipped file #{}: {}", i, e)}
            }
            //debug: leave test running so you can go check temp dir contents
            //while true{};
            //assert!(false, "unzipped_file_path was: {}", unzipped_file_path);

            //lazy comparison assumes files all contain the same string literal
            assert_eq!("here's some file contents", unzipped_contents);
            //clear contents of temp file to be written to again with .copy_to()
            //tmpfile.set_len(0);
        }
        let kill_result = process_handle.kill();
        match kill_result {
            Ok(o) => {println!("killed server")}
            //won't print if tests pass...
            Err(e) =>{println!("could not kill server: {}", e)}
        }

    }
    dir.close().expect("failed to delete temp file directory");
    archive_dir.close().expect("failed to delete archive directory");
/*//can kill server if duct process handle is no longer in scope
    let proc_num = cmd!("lsof", "-t", "-i:3000").read();
    let kill_result =cmd!("kill", proc_num.unwrap()).read();
    match kill_result {
        Ok(o) => {println!("successfully killed server: {}", o)}
        //this will not be printed to stdout if test passes...
        Err(e) => {println!("Warning!: failed to kill server: {}", e)}
    }
*/
}




