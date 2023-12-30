#![allow(dead_code)]

mod disk_info;
use std::fs;
use std::io::{stdin, stdout, Write};
use std::str;
use std::time::SystemTime;

use disk_info::virtual_disk::*;
use disk_info::*;


fn main() {
    // 是否从文件读取数据
    let mut virtual_disk = ui_load_dm_loop(SAVE_FILE_NAME);
    ui_loop(&mut virtual_disk);
}


const SAVE_FILE_NAME: &str = "./file_system";

const UI_HELP: &str = "\
\n==================================================\
\n           IvanD's Basic File System\
\n==================================================\
\nHelp:\
\n\tcd <dirname>: Change current dir.\
\n\tmkdir <dir name>: Create a new dir.\
\n\tls : List all files and dir in current dir.\
\n\tcat <filename>: Show the file content.\
\n\trm <filename>: Delete a file on disk.\
\n\tdiskinfo : Show some info about disk.\
\n\tsave : Save this virtual disk to file 'file-sys.vd'\
\n\texit : Exit the system. 
\n\
\nTesting:\
\n\ttest create: Create a random file to test.\
\n\
\nSystem Inner Function:\
\n\tfn create_file_with_data(&mut self, name: &str, data: &[u8])\
\n\tfn rename_file(&mut self, old: &str, new: &str)\
\n\tfn delete_file_by_name(&mut self, name: &str)\
\n\tfn read_file_by_name(&self, name: &str) -> Vec<u8>\
\n"; // UI主菜单

/// 使用交互式让用户选择是否从硬盘中加载DiskManager进行使用
fn ui_load_dm_loop(filename: &str) -> DiskInfo {
    let mut buf_str = String::new();
    loop {
        pinfo();
        print!("Do you want to try to load file-sys.vd? [Y/N] ");
        stdout().flush().unwrap();
        stdin().read_line(&mut buf_str).unwrap();
        let first_char = buf_str.as_str().trim().chars().next().unwrap();

        match first_char {
            'N' | 'n' => {
                pinfo();
                println!("Will not load vd file from disk.\n");

                break DiskInfo::new(None);
            }
            'Y' | 'y' => {
                pinfo();
                println!("Trying to load vd file from disk...\n");
                let data = fs::read(filename).unwrap();

                break bincode::deserialize(data.as_slice()).unwrap();
            }
            _ => {
                println!("\nIncorrect input.");
                continue;
            }
        };
    }
}

/// 一个简单的交互式界面。
fn ui_loop(virtual_disk: &mut DiskInfo) {
    // 交互界面
    println!("{}", UI_HELP);

    let mut buf_str = String::new();

    loop {
        // 清空buffer
        buf_str.clear();
        print!("> ");
        stdout().flush().unwrap();
        stdin().read_line(&mut buf_str).unwrap();
        // 去除首尾空格
        let command_line = String::from(buf_str.trim());

        // 分支-test
        if let Some(cl) = command_line.strip_prefix("test ") {
            // 分支-create
            if let Some(cl) = cl.strip_prefix("create") {
                let data = format!("File has been created at {:?} .", SystemTime::now());
                let cl_trim = cl.trim();
                let name = if cl_trim.is_empty() {
                    // 没有输入名字
                    format!("test-{}", (rand::random::<f32>() * 100_f32) as usize)
                } else {
                    // 输入了名字
                    cl_trim.to_string()
                };
                virtual_disk.create_file_with_data(name.as_str(), data.as_bytes());
            }
        } else if command_line.starts_with("help") {
            // 显示菜单
            println!("{}", UI_HELP);
        } else if command_line.starts_with("exit") {
            // 跳出循环，结束程序
            pinfo();
            println!("Exiting system...\n");
            break;
        } else if command_line.starts_with("save") {
            // 保存系统
            pinfo();
            println!("Saving...");
            let data = bincode::serialize(&virtual_disk).unwrap();
            fs::write(SAVE_FILE_NAME, data.as_slice()).unwrap();
            pinfo();
            println!("The virtual disk system has been saved.\n");
        } else if command_line.starts_with("ls") {
            // 列出目录文件
            println!("{}", virtual_disk.cur_directory);
        } else if let Some(name) = command_line.strip_prefix("cd ") {
            // 切换到当前目录的某个文件夹
            pinfo();
            println!("Set Location to: {} ...", name);
            virtual_disk.change_current_directory(name);
        } else if let Some(command_line) = command_line.strip_prefix("cat ") {
            // 显示文件内容
            let name = command_line.trim();
            let data = virtual_disk.read_file_by_name(name);
            println!("{}", str::from_utf8(data.as_slice()).unwrap());
        } else if let Some(command_line) = command_line.strip_prefix("mkdir ") {
            // 创建新文件夹
            let name = command_line.trim();
            virtual_disk.new_directory_to_disk(name).unwrap();
        } else if command_line.starts_with("diskinfo") {
            // 返回磁盘信息
            let (disk_size, num_used, num_not_used) = virtual_disk.get_disk_info();
            println!(
                "Disk sized {} Bytes, {} Bytes used, {} Bytes available.",
                disk_size,
                num_used * BLOCK_SIZE,
                num_not_used * BLOCK_SIZE
            );
        } else if let Some(command_line) = command_line.strip_prefix("rm ") {
            let name = command_line.trim();
            virtual_disk
                .delete_file_by_name(name)
                .expect("[ERROR]\tDELETE FILE FAILED!");
        } else if let Some(command_line) = command_line.strip_prefix("cp ") {
            // 复制文件
            let name: Vec<&str> = command_line.trim().split(" ").collect();
            if name.len() != 2 {
                println!("Parameter Error!");
                continue;
            }
            virtual_disk.copy_file_by_name(name[0], name[1]);
        } else if let Some(command_line) = command_line.strip_prefix("mv ") {
            // 重命名/移动文件
            let name: Vec<&str> = command_line.trim().split(" ").collect();
            if name.len() != 2 {
                println!("Parameter Error!");
                continue;
            }
            if name[1].contains("/") {
                // 移动，path暂时只支持相对路径
                virtual_disk.movie_file_by_name(name[0], name[1]);
            } else {
                // 重命名
                virtual_disk.rename_file_by_name(name[0], name[1]);
            }
        } else {
            println!("Unknown Command.");
        }
    }
}
