use std::io::Write;
use std::io::stdout;
use bytes::BufMut;

use crate::utils;
use crate::utils::{disable_raw_mode, enable_raw_mode};

use crate::trie::Trie;

pub struct Reader {
    command_tree: Trie
}

impl Reader {
    pub fn new () -> Self {
        Self { command_tree: Trie::new() }
    }

    pub fn update_trie <T>(&mut self, words: &Vec<T>) 
    where
        T: AsRef<str>
    {
        for word in words {
            self.command_tree.insert(word.as_ref());
        }
    }

    pub fn read_line (&self, prompt: &str) -> String {
        const STDIN_D: i32 = 0;

        let original = enable_raw_mode(STDIN_D);
        let _ = write!(stdout(), "{prompt}");
        let _ = stdout().flush();

        let mut bell = false;

        let mut buf = [0u8, 1];
        let mut input = Vec::<u8>::new();

        loop {
            let n = unsafe { libc::read(STDIN_D, buf.as_mut_ptr().cast(), 1) };

            if n <= 0 {
                break;
            }

            match buf[0] {
                b'\n' | b'\r' => {
                    print!("\n");
                    break;
                }
                0x7F => {
                    if !input.is_empty() {
                        input.pop();
                        print!("\x08 \x08");
                        let _ = stdout().flush();
                    }
                }
                b'\t' => {
                        let inp = String::from_utf8_lossy(&input).to_string();
                        let mut completions = self.command_tree.with_prefix(&inp);
                        completions.sort();

                        if completions.len() == 1 {
                            let out = &completions[0][inp.len()..];
                            input.put_slice(out.as_bytes());
                            input.push(b' ');

                            print!("{} ", out);
                        } else if completions.len() > 1 {
                            if bell {
                                print!("\n");
                                for comp in completions {
                                    print!("{comp}  ");
                                }
                                print!("\n");
                                
                                print!("{prompt}");
                                print!("{}", inp);
                            } else {
                                let lcp = utils::longest_common_prefix(&inp, &completions);

                                if lcp == inp {
                                    bell = true;
                                    print!("\x07");
                                } else {
                                    let out = &lcp[inp.len()..];
                                    input.put_slice(out.as_bytes());

                                    print!("{}", out);
                                }
                            }
                        } else {
                            print!("\x07");
                        }
                    
                    let _ = stdout().flush();
                    
                }
                byte => {
                    input.push(byte);
                    print!("{}", byte as char);
                    let _ = stdout().flush();
                }
            }
        }

        disable_raw_mode(STDIN_D, &original);
        String::from_utf8_lossy(&input).to_string()
    }
}

#[cfg(test)]
mod io_tests {
    use super::*;

    #[test]
    fn readline () {
        let reader = Reader::new();

        let input = reader.read_line("Hey: ");

        print!("{input}");
    }
}
