//he he he he cat metadata
use std::path::Path;
use std::{fs::{File, OpenOptions}, io::{BufReader, Read, Write}};
use html_parser::{HTMLError, parse_file, flaten_tree};
use html_parser::tag::IterTag;

const UNIX_DAY_JULIAN: u64 = 2440588;
const UNIX_EPOCH_DAY: u64 = 719_163;

#[derive(Debug)]
pub struct Cbmd {
    title: String,
    intro_words: String,
    path: String,
    publish_ts: u64,
}

impl Cbmd {
    pub fn new(mut title: String, mut intro_words: String, path: String, publish_ts: u64) -> Self {
        trim_newline(&mut title);
        trim_newline(&mut intro_words);
        Self {
            title,
            intro_words,
            path,
            publish_ts,
        }
    }

    pub fn from_html_file(path: &Path) -> Result<Self, HTMLError> {
        let tag_tree = match parse_file(path) {
            Err(e) => {
                println!("{e}");
                Err(e)?
            },
            Ok(tt) => tt,
        };
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
    
        let path = cut_down_full_path(path.to_str().unwrap()).to_string();
        let publish_date = mm_dd_yyyy_since_epoch(&publish_date);
    
        Ok(Cbmd::new(title, intro, path, publish_date))
    }

    pub fn from_meta_file(path: &Path) -> Result<Self, std::io::Error> {
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
        
        let mut path_len = [0_u8; 1];
        buf_reader.read_exact(&mut path_len)?;
        let mut path_bytes = vec![0_u8; path_len[0] as usize];
        buf_reader.read_exact(&mut path_bytes)?;
        let path = String::from_utf8_lossy(&path_bytes).to_string();

        let mut ts_bytes = [0_u8; 8];
        buf_reader.read_exact(&mut ts_bytes)?;
        let publish_ts = u64::from_le_bytes(ts_bytes);

        Ok(Self {
            title,
            intro_words,
            path,
            publish_ts,
        })
    }

    pub fn format_date(&self) -> String {
        let date = epoch_time_stamp_to_time(self.publish_ts);
        format!("{}/{}/{}", date.month(), date.day(), date.year())
    }

    pub fn write_to_file(self, out_path: &Path) -> Result<(), std::io::Error> {
        let buffer = self.serialize();
    
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(out_path)?;
    
        file.write_all(&buffer)?;
        file.flush()?;

        Ok(())
    }

    pub fn get_timestamp(&self) -> u64 {
        self.publish_ts
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn serialize(&self) -> Vec<u8> {
        let title_len = self.title.len();
        let words_len = self.intro_words.len();
        let path_len = self.path.len();

        let mut data: Vec<u8> = Vec::with_capacity(1 + title_len + 1 + words_len + 1 + path_len + 8);
        data.push(title_len as u8);
        data.extend_from_slice(self.title.as_bytes());
        data.push(words_len as u8);
        data.extend_from_slice(self.intro_words.as_bytes());
        data.push(path_len as u8);
        data.extend_from_slice(self.path.as_bytes());

        let ts_bytes = self.publish_ts.to_le_bytes();
        data.extend_from_slice(&ts_bytes);

        data
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

fn cut_down_full_path(s: &str) -> &str {
    &s[13..s.len()-5]
}

fn mm_dd_yyyy_since_epoch(date: &str) -> u64 {
    let date = date.trim();

    let month_day_year = date.split("/")
        .map(|num| num.parse::<i32>())
        .filter(|res| res.is_ok())
        .map(|u| u.unwrap())
        .collect::<Vec<i32>>();

    assert!(month_day_year.len() == 3);

    let month = match month_day_year[0] % 12 {
        0 => time::Month::December,
        1 => time::Month::January,
        2 => time::Month::February,
        3 => time::Month::March,
        4 => time::Month::April,
        5 => time::Month::May,
        6 => time::Month::June,
        7 => time::Month::July,
        8 => time::Month::August,
        9 => time::Month::September,
        10 => time::Month::October,
        11 => time::Month::November,
        _ => unreachable!(),
    };

    // this code is stolen from chrono but i didnt want that massive crate just for 
    // a time stamp so I got time and just implented the .timestap method and the
    // days_from_ce in the datelike trait. This also does not acount for leapseconds
    // but that only means this is 23 seconds behind and I belive this is in UTC

    let date = time::Date::from_calendar_date(month_day_year[2], month, month_day_year[1] as u8).unwrap();
    let mut year = date.year() - 1;
    let mut ndays_from_ce = 0;
    if year < 0 {
        let excess = 1 + (-year) / 400;
        year += excess * 400;
        ndays_from_ce -= excess * 146_097;
    }
    let div_100 = year / 100;
    ndays_from_ce += ((year * 1461) >> 2) - div_100 + (div_100 >> 2);
    ndays_from_ce += date.ordinal() as i32;
    let gregorian_day = u64::from(ndays_from_ce as u32);

    (gregorian_day - UNIX_EPOCH_DAY) * 86_400
}