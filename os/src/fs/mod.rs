//! File trait & inode(dir, file, pipe, stdin, stdout)
mod inode;
mod stdio;

use crate::mm::UserBuffer;

/// trait File for all file types
pub trait File: Send + Sync {
    /// the file readable?
    fn readable(&self) -> bool;
    /// the file writable?
    fn writable(&self) -> bool;
    /// read from the file to buf, return the number of bytes read
    fn read(&self, buf: UserBuffer) -> usize;
    /// write to the file from buf, return the number of bytes written
    fn write(&self, buf: UserBuffer) -> usize;
    /// get stat of the file
    fn stat(&self, st: &mut Stat);
}

use easy_fs::Stat;
pub use inode::{inode_link, inode_unlink, list_apps, open_file, OSInode, OpenFlags};
pub use stdio::{Stdin, Stdout};

// 还记得在ch6之前我们使用的loader.rs模块吗？
// lazy_static! {
//     static ref APP_NAMES: Vec<&'static str> = {
//         let num_app = get_num_app();
//         extern "C" {
//             fn _app_names();
//         }
//         let mut start = _app_names as usize as *const u8;
//         let mut v = Vec::new();
//         unsafe {
//             for _ in 0..num_app {
//                 let mut end = start;
//                 while end.read_volatile() != b'\0' {
//                     end = end.add(1);
//                 }
//                 let slice = core::slice::from_raw_parts(start, end as usize - start as usize);
//                 let str = core::str::from_utf8(slice).unwrap();
//                 v.push(str);
//                 start = end.add(1);
//             }
//         }
//         v
//     };
// }
//
// 之前就是这个模块提供了列出所有可执行程序的名字，还有根据可执行文件名字
// 查找可执行程序id的功能：
//
// pub fn get_num_app() -> usize {
//     extern "C" {
//         fn _num_app();
//     }
//     unsafe { (_num_app as usize as *const usize).read_volatile() }
// }
//
// pub fn get_app_data_by_name(name: &str) -> Option<&'static [u8]> {
//     let num_app = get_num_app();
//     (0..num_app)
//         .find(|&i| APP_NAMES[i] == name)
//         .map(get_app_data)
// }
//
// pub fn get_app_data(app_id: usize) -> &'static [u8] {
//     extern "C" {
//         fn _num_app();
//     }
//     let num_app_ptr = _num_app as usize as *const usize;
//     let num_app = get_num_app();
//     let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
//     assert!(app_id < num_app);
//     unsafe {
//         core::slice::from_raw_parts(
//             app_start[app_id] as *const u8,
//             app_start[app_id + 1] - app_start[app_id],
//         )
//     }
// }
//
// pub fn list_apps() {
//     println!("/**** APPS ****");
//     for app in APP_NAMES.iter() {
//         println!("{}", app);
//     }
//     println!("**************/");
// }
