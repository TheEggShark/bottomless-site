//he he he he cat metadata

use std::{fs::{File, OpenOptions}, io::{BufReader, Read, Write}};
use super::mm_dd_yyyy_since_epoch;
use html_parser::{HTMLError, parse_file, flaten_tree};
use html_parser::tag::IterTag;

const UNIX_DAY_JULIAN: u64 = 2440588;

#[derive(Debug)]
pub struct Cbmd {
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

    pub fn from_html_file(path: &str) -> Result<Self, HTMLError> {
        let tag_tree = parse_file(path)?;
        let meta_tags = flaten_tree(tag_tree)
            .into_iter()
            .filter(|t| t.get_name() == "meta")
            .collect::<Vec<IterTag>>();
    
        let mut publish_date = String::new();
        let mut title = String::new();
        let mut intro = String::new();
    
        for tag in meta_tags {
            if let Some(attribute) = tag.get_attribute("publish-date") {
                publish_date = attribute.to_string();
                continue;
            }
    
            if let Some(attribute) = tag.get_attribute("title") {
                title = attribute.to_string();
                continue;
            }
    
            if let Some(attribute) = tag.get_attribute("intro") {
                intro = attribute.to_string();
            }
        }
    
        let publish_date = mm_dd_yyyy_since_epoch(&publish_date);
    
        Ok(Cbmd::new(title, intro, publish_date))
    }

    pub fn from_file(path: &str) -> Result<Self, std::io::Error> {
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

    pub fn format_date(&self) -> String {
        let date = epoch_time_stamp_to_time(self.publish_ts);
        format!("{}/{}/{}", date.month(), date.day(), date.year())
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

fn epoch_time_stamp_to_time(timestamp: u64) -> time::Date {
    // found this online just rounded the thing up for rust dont care about
    // second accuracy just need month day year!
    // return ( timestamp / 86400.0 ) + 2440587.5;
    let julian = (timestamp / 86400) + UNIX_DAY_JULIAN;
    time::Date::from_julian_day(julian as i32).unwrap()
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}