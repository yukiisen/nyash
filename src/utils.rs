use std::ffi::{ CStr, CString };
use std::env::split_paths;
use std::os::fd::RawFd;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::io;
use std::str::Chars;

use anyhow::Result;

use crate::error::ReadDirError;

pub fn get_environment (var: &str) -> Option<&str> {
    let var = format!("{var}\0");
    unsafe  {
        let ptr = libc::getenv(var.as_ptr().cast());
        if !ptr.is_null() {
            let c_env = CStr::from_ptr(ptr);

            c_env.to_str().ok()
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct DirIter {
    dir: *mut libc::DIR
}

impl DirIter {
    pub fn from_dir(dir: *mut libc::DIR) -> Self {
        Self { dir }
    }
}

impl Iterator for DirIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let entry = libc::readdir(self.dir);
            if entry.is_null() {
                libc::closedir(self.dir);
                None
            } else {
                let dname = (*entry).d_name.as_ptr();
                Some(CStr::from_ptr(dname).to_string_lossy().to_string())
            }
        }
    }
}

pub fn open_file <T: AsRef<str>>(path: T) -> i32 {
    use libc::{ O_RDWR, O_CREAT, O_APPEND };
    use libc::{ S_IRUSR, S_IWUSR, S_IRGRP, S_IROTH };
    let path = CString::new(path.as_ref()).expect("Path contained a null byte");
    unsafe {
        libc::open(path.as_ptr().cast(), O_RDWR | O_CREAT | O_APPEND, S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH)
    }
}

pub fn read_directory<P> (dirname: P) -> Result<DirIter> 
where 
    P: AsRef<Path>,
{
    let dirname = dirname.as_ref().as_os_str();
    let c_dirname = CString::new(dirname.as_bytes())?;

    unsafe {
        let dir = libc::opendir(c_dirname.as_ptr());
        if dir.is_null() {
            let err = ReadDirError::OpenDirError(format!("failed to open {:?}: found a null pointer", dirname.to_str()));
            return Err(err.into());
        }

        Ok(DirIter::from_dir(dir))
    }
}

pub fn get_executable (cmd: &str) -> Option<String> {
    let path = get_environment("PATH").unwrap_or("");
    let dirs =  split_paths(path);

    for dir in dirs {
        if let Ok(iter) = read_directory(&dir) {
            let executables: Vec<String> = iter.filter(|e| e != "." && e != "..").collect();

            if executables.contains(&cmd.to_string()) {
                return Some(dir.join(cmd).to_string_lossy().to_string());
            }
        }
    }

    None
}

pub fn get_system_binaries () -> Vec<String> {
    let path = get_environment("PATH").unwrap_or("");
    let dirs = split_paths(path);
    let mut res = Vec::new();

    for dir in dirs {
        if let Ok(iter) = read_directory(&dir) {
            let mut executables: Vec<String> = iter.filter(|e| e != "." && e != "..").collect();

            res.append(&mut executables);
        }
    }

    res
}

pub fn longest_common_prefix <T: AsRef<str>>(word: &str, completions: &Vec<T>) -> String {
    let mut i = word.len();
    let mut lcp = String::from(word);
    let mut completions = completions.iter().map(|e| e.as_ref().chars()).collect::<Vec<Chars<'_>>>();

    for comp in &mut completions {
        comp.nth(i - 1).unwrap();
    }

    loop {
        let mut ch = None::<char>;
        for comp in &mut completions {
            if let Some(character) = comp.next() {
                if ch.is_none() {
                    ch = Some(character);
                } else if character != ch.unwrap() {
                    return lcp;
                };
            } else {
                return lcp;
            }
        };

        lcp.push(ch.unwrap());

        i += 1;
    }
}

pub fn get_pwd () -> String {
    // a buffer to store pwd since the function can't allocate memory.
    const SIZE: usize = 4096;
    let mut buf = vec![0u8; SIZE];

    unsafe {
        let ptr = libc::getcwd(buf.as_mut_ptr() as *mut libc::c_char, SIZE);

        if ptr.is_null() {
            "".to_string()
        } else {
            let pwd = CStr::from_ptr(ptr).to_string_lossy();
            pwd.to_string()
        }
    }
}

/// Redirects current process's io streams to a separate file
pub fn redirect_io (file: &str, flags: i32, stream_fd: RawFd) -> Result<()> {
    let c_file = CString::new(file)?;
    let fd = unsafe { libc::open(
            c_file.as_ptr(), 
            flags, 
            0o644
        ) 
    };

    if fd < 0 {
        return Err(io::Error::new(io::ErrorKind::Other, "failed to get file descriptor").into());
    }

    unsafe {
        libc::dup2(fd, stream_fd);
        libc::close(fd);
    }

    Ok(())
}

pub fn enable_raw_mode (fd: i32) -> libc::termios {
    use libc::{ TCSANOW, VMIN, ECHO, ICANON, VTIME };

    unsafe {
        let mut term = std::mem::zeroed();
        libc::tcgetattr(fd, &mut term);

        let original = term;

        term.c_lflag &= !(ICANON | ECHO);
        term.c_cc[VMIN] = 1;
        term.c_cc[VTIME] = 0;

        libc::tcsetattr(fd, TCSANOW, &term);
        original
    }
}

pub fn disable_raw_mode(fd: i32, original: &libc::termios) {
    unsafe {
        libc::tcsetattr(fd, libc::TCSANOW, original);
    }
}

#[derive(Debug, Clone)]
pub struct PipeLine {
    pub fd_t: i32,
    pub fds: [i32; 2]
}

#[cfg(test)]
mod utility_tests {
    use super::*;

    #[test]
    fn path_tester () {
        let path = get_environment("PATH").unwrap_or("");
        let term = get_environment("TERM").unwrap_or("");

        assert_eq!(path.to_string(), std::env::var("PATH").unwrap());
        assert_eq!(term.to_string(), std::env::var("TERM").unwrap());
    }

    #[test]
    fn readdir_test () {
        let entries = read_directory("/home/yuki/").unwrap();
        let files = entries.collect::<Vec<String>>();

        assert!(files.contains(&".bashrc".to_string()));
    }

    #[test]
    fn get_exec_test() {
        let lazysql = get_executable("lazysql");
        let echo = get_executable("echo");
        let sv = get_executable("sv");
        let vim = get_executable("vim");

        assert!(echo.is_some(), "`echo` should be found in PATH");
        assert!(vim.is_some(), "`vim` should be found in PATH");

        println!("lazysql: {:?}", lazysql);
        println!("sv: {:?}", sv);

        if let Some(ref path) = echo {
            assert!(std::path::Path::new(path).exists(), "`echo` path should exist");
        }

        if let Some(ref path) = vim {
            assert!(std::path::Path::new(path).exists(), "`vim` path should exist");
        }
    }

    #[test]
    fn get_lcp () {
        let word = "clear-";
        let completions = ["clear-fbo", "clear-fbo-scissor", "clear-fbo-tex"];

        let lcp = longest_common_prefix(word, &completions.to_vec());

        dbg!(&lcp);

        assert_eq!(lcp, "clear-fbo");
    }
}
