//he he he he cat metadata
use crate::parser;

use std::{fs::{File, OpenOptions}, io::{BufReader, Read, Write}};

struct Cbmd {
    title: String,
    intro_words: String,
    publish_ts: u64,
}

impl Cbmd {
    pub fn new(mut title: String, mut intro_words: String, publish_ts: u64) -> Self {
        trim_newline(&mut title);
        trim_newline(&mut intro_words);
        Self {
            title,
            intro_words,
            publish_ts,
        }
    }

    pub fn from_file(path: &str) -> Result<Self, std::io::Error>{
        let file = File::open(path)?;
        let mut buf_reader = BufReader::new(file);
        
        let mut title_len = [0_u8; 1];
        buf_reader.read_exact(&mut title_len)?;

        let mut title_bytes = vec![0_u8; title_len[0] as usize];
        buf_reader.read_exact(&mut title_bytes)?;
        let title = String::from_utf8_lossy(&title_bytes).to_string();

        let mut intro_word_len = [0_u8; 1];
        buf_reader.read_exact(&mut intro_word_len)?;

        let mut intro_word_bytes = vec![0_u8; intro_word_len[0] as usize];
        buf_reader.read_exact(&mut intro_word_bytes)?;
        let intro_words = String::from_utf8_lossy(&intro_word_bytes).to_string();
        
        let mut ts_bytes = [0_u8; 8];
        buf_reader.read_exact(&mut ts_bytes)?;
        let publish_ts = u64::from_le_bytes(ts_bytes);

        Ok(Self {
            title,
            intro_words,
            publish_ts,
        })
    }

    pub fn write_to_file(self, out_path: &str) -> Result<(), std::io::Error> {
        let t_bytes = self.title.as_bytes();
        let t_len = t_bytes.len();
        assert!(t_len < 256);
        let i_bytes = self.intro_words.as_bytes();
        let i_len = i_bytes.len();
        assert!(i_len < 256);
        let ts_bytes = self.publish_ts.to_le_bytes();
    
        let mut buffer: Vec<u8> = Vec::with_capacity(t_len + i_len + 8);
        buffer.push(t_len as u8);
        buffer.extend_from_slice(t_bytes);
        buffer.push(i_len as u8);
        buffer.extend_from_slice(i_bytes);
        buffer.extend_from_slice(&ts_bytes);
    
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(out_path)?;
    
        file.write_all(&buffer)?;
        file.flush()?;

        Ok(())
    }
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}