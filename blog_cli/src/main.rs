mod cbmd;

use std::{fs::OpenOptions, io::Write};
const UNIX_EPOCH_DAY: u64 = 719_163;
const UNIX_DAY_JULIAN: u64 = 2440588;


fn main() {

    let ts = mm_dd_yyyy_since_epoch("08/15/2006");
    epoch_time_stamp_to_time(ts);

    println!("Welcome to Charlies Blog Metadata(.CBMD) creater");
    println!("Give the title of the blog");
    let mut title = String::new();
    std::io::stdin().read_line(&mut title).unwrap();

    println!("Give the intro words!");
    let mut intro = String::new();
    std::io::stdin().read_line(&mut intro).unwrap();

    println!("what is the publish date of the article in mm-dd-yyyy");
    let mut date = String::new();
    std::io::stdin().read_line(&mut date).unwrap();
    let ts = mm_dd_yyyy_since_epoch(&date);

    let mut out_path = String::new();
    println!("where should the metada be outputed");
    std::io::stdin().read_line(&mut out_path).unwrap();
    println!("{title}{intro}{out_path}");

    // serilze time
    create_file(title.trim(), intro.trim(), &out_path.trim(), ts);
}

fn create_file(title: &str, intro: &str, path: &str, timestamp: u64) {
    let t_bytes = title.as_bytes();
    let t_len = t_bytes.len();
    assert!(t_len < 256);
    let i_bytes = intro.as_bytes();
    let i_len = i_bytes.len();
    assert!(i_len < 256);
    let ts_bytes = timestamp.to_le_bytes();

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
        .open(path)
        .unwrap();

    file.write_all(&buffer).unwrap();
    file.flush().unwrap();
}

fn epoch_time_stamp_to_time(timestamp: u64) -> time::Date {
    // found this online just rounded the thing up for rust dont care about
    // second accuracy just need month day year!
    // return ( timestamp / 86400.0 ) + 2440587.5;
    let julian = (timestamp / 86400) + UNIX_DAY_JULIAN;
    time::Date::from_julian_day(julian as i32).unwrap()
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