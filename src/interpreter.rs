use std::collections::HashMap;
use std::process;
use std::ffi::CString;
use std::io;

use anyhow::{Error, Result};

use libc::{ O_RDONLY, O_WRONLY, O_CREAT, O_TRUNC, O_APPEND };

use crate::utils::{self, PipeLine, get_environment, get_executable, get_pwd, redirect_io};

#[derive(Debug, Default)]
pub struct Interpreter {
    builtins: HashMap<&'static str, fn(&[&str], &Interpreter)>,
    shell_commands: HashMap<&'static str, fn(&[&str], &Interpreter)>,
    pub history: i32
}

impl Interpreter {
    pub fn new(history: &str) -> Self {
        let mut inter = Interpreter { history: utils::open_file(history), ..Interpreter::default() };

        inter.shell_commands.insert("exit", |argv: &[&str], _| process::exit(argv.first().unwrap_or(&"0").parse::<i32>().unwrap_or(0)));
        inter.builtins.insert("echo", |argv: &[&str], _| println!("{}", argv.join(" ")));
        inter.builtins.insert("type", |argv: &[&str], interpreter: &Interpreter| {
            if let Some(cmd) = argv.first() {
                if interpreter.get_builtins().contains(&cmd) {
                    println!("{cmd} is a shell builtin")
                }
                else if let Some(path) = get_executable(cmd) {
                    println!("{cmd} is {path}");
                }
                else {
                    println!("{cmd}: not found")
                };
            }
        });

        inter.builtins.insert("pwd", |_, _| {
            let pwd = get_pwd();
            if pwd.is_empty() {
                println!("Error excuting getcwd");
            } else {
                println!("{pwd}");
            }
        });

        inter.builtins.insert("history", |_, inter| {

        });

        inter.shell_commands.insert("cd", |argv, _| {
            let mut dir = argv.first().unwrap_or(&"~").to_string();

            if dir == "~" {
                dir = get_environment("HOME").unwrap_or("/root").to_string();
            } else if !dir.starts_with("/") {
                let parts: Vec<&str> = dir.split("/").collect();
                let pwd = get_pwd();
                let mut res: Vec<&str> = pwd.split("/").filter(|e| !e.is_empty()).collect();

                for part in parts {
                    if part == "." { continue; }
                    if part == ".." {
                        res.pop();
                        continue;
                    }

                    res.push(part);
                }

                dir = res.join("/");
                dir = format!("/{dir}");
            }

            let Ok(path) = CString::new(dir.as_str()) else {
                println!("Failed to create path from {}", dir);
                return;
            };

            unsafe {
                if libc::chdir(path.as_ptr()) != 0 {
                    println!("cd: {dir}: No such file or directory");
                }
            }
        });

        inter
    }

    pub fn get_builtins (&self) -> Vec<&&str> {
        let mut vec: Vec<&&str> = self.builtins.keys().collect();

        for key in self.shell_commands.keys() {
            vec.push(key);
        }
        
        vec
    }

    pub fn interpret_command (&self, cmd: &str, argv: &Vec<&str>) -> bool {
        let commands = argv.split(|argv| argv == &"|").collect::<Vec<&[&str]>>();

        if commands.len() == 1 {
            if let Some(executor) = self.shell_commands.get(cmd) {
                executor(&argv[1..], self);
                return true;
            }

            if let Some(path) = get_executable(cmd) {
                match self.exec_command(&path, argv, None) {
                    Err(error) => eprintln!("error running {cmd}: {error}"),
                    Ok(pid) => { Self::wait_process(pid); }
                };
                return true;
            }

            if self.builtins.contains_key(cmd) {
                match self.exec_command(cmd, argv, None) {
                    Err(error) => eprintln!("error running {cmd}: {error}"),
                    Ok(pid) => { Self::wait_process(pid); }
                };
                return true;
            }


            return false;
        }


        let mut all_fds = Vec::new();
        let mut pids = Vec::new();
        let num_pipes = commands.len() - 1;
        
        for _ in 0..num_pipes {
            let mut fds = [0; 2];
            unsafe { libc::pipe(fds.as_mut_ptr()); }
            all_fds.push(fds);
        }

        for ( idx, argv ) in commands.iter().enumerate() {
            if argv.is_empty() { return false; };
            let cmd = argv.first().unwrap();

            let mut pipeline = Vec::new();

            if idx > 0 {
                pipeline.push(PipeLine {
                    fd_t: libc::STDIN_FILENO,
                    fds: all_fds[idx - 1],
                });
            }

            if idx < all_fds.len() {
                pipeline.push(PipeLine {
                    fd_t: libc::STDOUT_FILENO,
                    fds: all_fds[idx],
                });
            }

            if let Some(path) = get_executable(cmd) {
                match self.exec_command(&path, argv, Some((pipeline, &all_fds))) {
                    Err(error) => eprintln!("error running {cmd}: {error}"),
                    Ok(pid) => { pids.push(pid); }
                };

                continue;
            }

            if self.builtins.contains_key(cmd) {
                match self.exec_command(cmd, argv, Some((pipeline, &all_fds))) {
                    Err(error) => eprintln!("error running {cmd}: {error}"),
                    Ok(pid) => { pids.push(pid); }
                };

                continue;
            }

            // this is not likely to happen during tests
            for [ read, write ] in &all_fds {
                unsafe {
                    libc::close(*read);
                    libc::close(*write);
                }
            }

            return false;
        }

        for [ read, write ] in &all_fds {
            unsafe {
                libc::close(*read);
                libc::close(*write);
            }
        }

        for pid in pids {
            Self::wait_process(pid);
        }

        true
    }

    pub fn wait_process (pid: i32) -> i32 {
        let mut status = 0;
        unsafe {
            libc::waitpid(pid, &mut status, 0);
        };
        status
    }

    pub fn exec_command (&self, cmd: &str, argv: &[&str], pipe: Option<(Vec<utils::PipeLine>, &Vec<[i32; 2]>)>) -> Result<i32> {
        unsafe {
            let pid = libc::fork();

            match pid {
                0 => {
                    // this is a child process!
                    
                    if let Some((pipes, all_fds)) = pipe {
                        for pipe in pipes {
                            match pipe.fd_t {
                                libc::STDOUT_FILENO => { 
                                    libc::dup2(pipe.fds[1], pipe.fd_t); 
                                    libc::close(pipe.fds[0]);
                                },
                                libc::STDIN_FILENO => {
                                    libc::dup2(pipe.fds[0], pipe.fd_t); 
                                    libc::close(pipe.fds[1]);
                                },
                                _ => {}
                            }
                        }

                        for [ r, w ] in all_fds {
                            libc::close(*w);
                            libc::close(*r);
                        }
                    }
                    
                    let r_argv = argv;
                    let mut argv = Vec::new();
                    let mut redirections = Vec::<(&str, &str)>::new();

                    let mut iter = r_argv.iter();

                    while let Some(&arg) = iter.next() {
                        match arg {
                            ">" | "<" | ">>" | "1>" | "2>" | "&>" | "1>>" | "2>>" => {
                                let Some(file) = iter.next() else { 
                                    return Err(io::Error::new(io::ErrorKind::Other, "Syntax error").into()); 
                                };

                                redirections.push(( arg, file ));
                            }
                            _ => {
                                argv.push(arg);
                            }
                        }
                    };

                    for (re, file) in redirections {
                        match re {
                            ">" | "1>" => {
                                let _ = redirect_io(file, O_WRONLY | O_TRUNC | O_CREAT, 1);
                            }
                            "<" => {
                                let _ = redirect_io(file, O_RDONLY, 0);
                            }
                            ">>" | "1>>" => {
                                let _ = redirect_io(file, O_WRONLY | O_CREAT | O_APPEND, 1);
                            }
                            "2>" => {
                                let _ = redirect_io(file, O_WRONLY | O_TRUNC | O_CREAT, 2);
                            }
                            "2>>" => {
                                let _ = redirect_io(file, O_WRONLY | O_CREAT | O_APPEND, 2);
                            }
                            "&>" => {
                                redirect_io(file, O_WRONLY | O_TRUNC | O_CREAT, 1).unwrap();
                                libc::dup2(1, 2);                                
                            }
                            _ => {}
                        }
                    }

                    if let Some(executor) = self.builtins.get(cmd) {
                        executor(&argv[1..], self);
                        process::exit(0);
                    }

                    // child process
                    let exec_path = CString::new(cmd)?;
                    let c_args: Vec<CString> = argv.iter()
                        .map(|&s| CString::new(s).unwrap())
                        .collect();

                    let mut argv: Vec<*const i8> = c_args.iter()
                        .map(|s| s.as_ptr())
                        .collect();

                    argv.push(std::ptr::null());

                    libc::execv(exec_path.as_ptr(), argv.as_ptr());

                    // we shouldn't normally reach here.
                    panic!("We're cooked man!");
                },
                pid if pid > 0 => { 
                    Ok(pid)
                },
                _ => {
                    let error = io::Error::new(io::ErrorKind::Other, format!("failed to spawn process {}", cmd));
                    Err(Error::from(error))
                }
            }
        }
    } 
}
