use std::time::UNIX_EPOCH;
const UNIX_EPOCH_DAY: u64 = 719_163;


fn main() {

    let ts = mm_dd_yyyy_since_epoch("08/15/2006");
    epoch_time_stamp_to_time(ts);

    println!("Welcome to Charlies Blog Metadata(.CBMD) creater");
    println!("Give the title of the blog");
    let mut title = String::new();
    std::io::stdin().read_line(&mut title).unwrap();
    println!("give me the path of the file!");
    let mut file_path = String::new();
    std::io::stdin().read_line(&mut file_path).unwrap();
    println!("Give the intro words!");
    let mut intro = String::new();
    std::io::stdin().read_line(&mut intro).unwrap();
    println!("{title}\n{file_path}\n{intro}");
}

fn file_path_to_seconds_since_epoch(path: &str) -> Result<u64, std::io::Error> {
    let path = std::path::Path::new(path);
    let medata_data = path.metadata()?.modified()?;
    let time_since_epoch = medata_data.duration_since(UNIX_EPOCH)
        .expect("file before UNIX EPOCH")
        .as_secs();

    Ok(time_since_epoch)
}

fn epoch_time_stamp_to_time(timestamp: u64) -> time::Date {
    // found this online just rounded the thing up for rust dont care about
    // second accuracy just need month day year!
    // return ( timestamp / 86400.0 ) + 2440587.5;
    let julian = (timestamp / 86400) + 2440588;
    time::Date::from_julian_day(julian as i32).unwrap()
}

fn mm_dd_yyyy_since_epoch(date: &str) -> u64 {
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

//taken from chrono which was taken from num crate 
fn div_rem(rhs: u64, other: u64) -> (u64, u64) {
    (rhs / other, rhs % other)
}

fn div_floor(rhs: u64, other: u64) -> u64 {
    // Algorithm from [Daan Leijen. _Division and Modulus for Computer Scientists_,
    // December 2001](http://research.microsoft.com/pubs/151917/divmodnote-letter.pdf)
    let (d, r) = div_rem(rhs, other);
    if (r > 0 && other < 0) || (r < 0 && other > 0) {
        d - 1
    } else {
        d
    }
}

fn mod_floor(rhs: u64, other: u64) -> u64 {
    // Algorithm from [Daan Leijen. _Division and Modulus for Computer Scientists_,
    // December 2001](http://research.microsoft.com/pubs/151917/divmodnote-letter.pdf)
    let r = rhs % other;
    if (r > 0 && other < 0) || (r < 0 && other > 0) {
        r + other
    } else {
        r
    }
}