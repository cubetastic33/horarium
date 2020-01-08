use super::Timetable;

use chrono::prelude::*;
use prettytable::Table;
use reqwest::header::*;
use scraper::{Html, Selector};

const URL: &str = "https://fiitjeelogin.com";

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"));
    headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate, br"));
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.5"));
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"));
    headers.insert(HOST, HeaderValue::from_static("fiitjeelogin.com"));
    headers.insert(ORIGIN, HeaderValue::from_static("https://fiitjeelogin.com"));
    headers.insert(REFERER, HeaderValue::from_static("https://fiitjeelogin.com/"));
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64; rv:72.0) Gecko/20100101 Firefox/72.0"));
    headers
}

fn construct_payload(client: &reqwest::Client, username: &str, password: &str) -> Result<[(&'static str, String);11], failure::Error> {
    let mut response = client.get("https://fiitjeelogin.com/StartPage.aspx").headers(construct_headers()).send()?;
    let login_page = Html::parse_document(&response.text()?);

    let mut payload = [
        ("__EVENTTARGET", String::new()),
        ("__EVENTARGUMENT", String::new()),
        ("__LASTFOCUS", String::new()),
        ("__VIEWSTATE", String::new()),
        ("__VIEWSTATEGENERATOR", String::new()),
        ("__EVENTVALIDATION", String::new()),
        ("username", username.to_owned()),
        ("password", password.to_owned()),
        ("btnlogin", String::from("Log+in")),
        ("txtUsername", String::new()),
        ("txtemailid", String::new())
    ];

    for i in 3..6 {
        let selector = Selector::parse(&("#".to_string() + payload[i].0)).unwrap();
        let value = match login_page.select(&selector).next() {
            Some(x) => x.value().attr("value").unwrap(),
            None => return Err(failure::err_msg(format!("#{} was not found", payload[i].0))),
        };
        payload[i].1 = String::from(value);
    }

    Ok(payload)
}

fn construct_payload_for_enrollment_no(client: &reqwest::Client, enrollment_no: &'static str) -> Result<Vec<(&'static str, String)>, failure::Error> {
    let mut response = client.get("https://fiitjeelogin.com/Default.aspx").headers(construct_headers()).send()?;
    let page = Html::parse_document(&response.text()?);

    let mut payload = vec![
        ("__EVENTTARGET", String::from("ct100$ddlEnrollNo")),
        ("__EVENTARGUMENT", String::new()),
        ("__LASTFOCUS", String::new()),
        ("__VIEWSTATE", String::new()),
        ("__VIEWSTATEGENERATOR", String::new()),
        ("__EVENTVALIDATION", String::new()),
        ("ctl00$ddlEnrollNo", String::from(enrollment_no)),
    ];

    for i in 3..6 {
        let selector_string = "#".to_string() + payload[i].0;
        let selector = Selector::parse(&selector_string).unwrap();
        let value = match page.select(&selector).next() {
            Some(x) => x.value().attr("value").unwrap(),
            None => return Err(failure::err_msg(format!("#{} was not found", payload[i].0))),
        };
        println!("{}", value);
        payload[i].1 = String::from(value);
    }

    Ok(payload)
}

fn construct_payload_for_timetable(client: &reqwest::Client) -> Result<Vec<(&'static str, String)>, failure::Error> {
    let mut response = client.get("https://fiitjeelogin.com/StudentTimeTable.aspx").headers(construct_headers()).send()?;
    let timetable_page = Html::parse_document(&response.text()?);

    let now = Local::today().naive_local();
    let start_date = now - chrono::Duration::days(now.weekday().num_days_from_sunday() as i64);
    let end_date = now + chrono::Duration::weeks(100);

    let mut payload = vec![
        ("__EVENTTARGET", String::new()),
        ("__EVENTARGUMENT", String::new()),
        ("__LASTFOCUS", String::new()),
        ("__VIEWSTATE", String::new()),
        ("__VIEWSTATEGENERATOR", String::new()),
        ("__EVENTVALIDATION", String::new()),
        ("ctl00$cphMaster$Accordion1_AccordionExtender_ClientState", String::new()),
        ("ctl00$cphMaster$btn_Filter", String::new()),
        ("ctl00$cphMaster$txt_StartDate", start_date.format("%e/%_m/%Y").to_string().replace(" ", "")),
        ("ctl00$cphMaster$txt_EndDate", end_date.format("%e/%_m/%Y").to_string().replace(" ", "")),
        ("__ASYNCPOST", String::from("true")),
    ];

    for i in 3..8 {
        let selector_string = "*[name=\"".to_string() + payload[i].0 + "\"]";
        let selector = Selector::parse(&selector_string).unwrap();
        let value = match timetable_page.select(&selector).next() {
            Some(x) => x.value().attr("value").unwrap(),
            None => return Err(failure::err_msg(format!("#{} was not found", payload[i].0))),
        };
        payload[i].1 = String::from(value);
    }

    Ok(payload)
}

fn minutes_to_string(minutes: usize) -> String {
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

fn fix_codes(code: &str, double_classes: bool) -> String {
    String::from(match code {
        "MJG1" | "MJGA" => "MJG",
        "BJK" => if double_classes { "BJK / IRN" } else { "BJK" }
        "BJS2" => "BJS",
        "EVL1" => "EVL",
        "IRN" => if double_classes { "BJK / IRN" } else { "IRN" }
        s => s
    })
}

// Function to convert class codes to subject names
fn convert_code_to_subject(class_code: &str) -> String {
    String::from(match class_code {
        "MPV" | "MJG" | "MVJ" | "MMK" | "MRJ" | "MRPK" => "Math",
        "BGD" | "BJK" => "Zoology",
        "BJS" => "Botany",
        "CRA" | "CVR" | "CRM" | "CRSA" | "CUNN" => "Chemistry",
        "PSK" | "PGR" | "PAA" | "PRB" | "PCM" | "PSOM" => "Physics",
        "EVL" | "ESB" | "EHM" | "EAB" => "English",
        "IRN" | "IAP" => "Computer Science",
        "BJK / IRN" => "Bio / CS",
        "PET" => "Games",
        s => s
    })
}

fn extract_timetable(source: &str, double_classes: bool) -> Result<Vec<(String, Vec<(String, String, String, String)>)>, failure::Error> {
    let mut timetables = Vec::new();
    let timetable_page = Html::parse_fragment(source);
    let selector = Selector::parse("table.CSSTableGenerator").unwrap();
    let tables = timetable_page.select(&selector);
    for table in tables {
        let date = timetable_page.select(&Selector::parse(&("#".to_string() + &table.value().attr("id").unwrap().replace("Content", "Header"))).unwrap()).next().unwrap().inner_html().trim().to_owned();
        println!("\n{}", date);
        let mut print_table = Table::new();
        let mut timetable = Vec::new();
        for row in table.select(&Selector::parse("tr:not(:first-child)").unwrap()) {
            let time_range = row.select(&Selector::parse("td:nth-child(2)").unwrap()).next().unwrap().inner_html().replace('\n', "");

            let mut class_code = row.select(&Selector::parse("td:nth-child(3)").unwrap()).next().unwrap().inner_html();
            if class_code.len() == 0 {
                class_code = row.select(&Selector::parse("td:nth-child(4)").unwrap()).next().unwrap().inner_html();
            }
            class_code = fix_codes(&class_code, double_classes).trim().to_owned();

            let times: Vec<_> = time_range.split("-").collect();
            let mut minutes1 = times[0].split(":").collect::<Vec<_>>()[0].parse::<usize>()? * 60 + times[0].split(":").collect::<Vec<_>>()[1].parse::<usize>()?;
            let minutes2 = times[1].split(":").collect::<Vec<_>>()[0].parse::<usize>()? * 60 + times[1].split(":").collect::<Vec<_>>()[1].parse::<usize>()?;
            let mut difference = minutes2 - minutes1;
            if difference > 60 {
                // If we need to split the times
                if &times[0] == &"08:40" {
                    print_table.add_row(row!["08:40 - 09:40", class_code]);
                    print_table.add_row(row!["09:40 - 10:40", class_code]);
                    timetable.push((String::from("08:40 - 09:40"), String::from("08:40&nbsp;AM - 09:40&nbsp;AM"), class_code.clone(), convert_code_to_subject(&class_code)));
                    timetable.push((String::from("09:40 - 10:40"), String::from("09:40&nbsp;AM - 10:40&nbsp;AM"), class_code.clone(), convert_code_to_subject(&class_code)));
                    if difference > 140 {
                        // If more than 3 periods were covered
                        print_table.add_row(row!["10:50 - 11:50", class_code]);
                        print_table.add_row(row!["11:50 - 12:50", class_code]);
                        timetable.push((String::from("10:50 - 11:50"), String::from("10:50&nbsp;AM - 11:50&nbsp;AM"), class_code.clone(), convert_code_to_subject(&class_code)));
                        timetable.push((String::from("11:50 - 12:50"), String::from("11:50&nbsp;AM - 12:50&nbsp;PM"), class_code.clone(), convert_code_to_subject(&class_code)));
                    }
                } else if &times[0] == &"14:50" {
                    print_table.add_row(row!["14:50 - 15:30", class_code]);
                    print_table.add_row(row!["15:30 - 16:20", class_code]);
                    timetable.push((String::from("14:50 - 15:30"), String::from("02:50&nbsp;PM - 02:30&nbsp;PM"), class_code.clone(), convert_code_to_subject(&class_code)));
                    timetable.push((String::from("15:30 - 16:20"), String::from("03:30&nbsp;PM - 04:20&nbsp;PM"), class_code.clone(), convert_code_to_subject(&class_code)));
                } else if &times[0] == &"13:20" {
                    while difference > 30 {
                        print_table.add_row(row![minutes_to_string(minutes1) + " - " + &minutes_to_string(minutes1 + 40), class_code.clone()]);
                        let start_time_12h = NaiveTime::parse_from_str(&minutes_to_string(minutes1), "%H:%M")?.format("%I:%M&nbsp;%p");
                        let finish_time_12h = NaiveTime::parse_from_str(&minutes_to_string(minutes1 + 40), "%H:%M")?.format("%I:%M&nbsp;%p");
                        let time_range_12h = format!("{} - {}", start_time_12h, finish_time_12h);
                        timetable.push((minutes_to_string(minutes1) + " - " + &minutes_to_string(minutes1 + 40), time_range_12h, class_code.clone(), convert_code_to_subject(&class_code)));
                        minutes1 += 40;
                        difference -= 40;
                    }
                } else {
                    while difference >= 60 {
                        print_table.add_row(row![minutes_to_string(minutes1) + " - " + &minutes_to_string(minutes1 + 60), class_code.clone()]);
                        let start_time_12h = NaiveTime::parse_from_str(&minutes_to_string(minutes1), "%H:%M")?.format("%I:%M&nbsp;%p");
                        let finish_time_12h = NaiveTime::parse_from_str(&minutes_to_string(minutes1 + 60), "%H:%M")?.format("%I:%M&nbsp;%p");
                        let time_range_12h = format!("{} - {}", start_time_12h, finish_time_12h);
                        timetable.push((minutes_to_string(minutes1) + " - " + &minutes_to_string(minutes1 + 60), time_range_12h, class_code.clone(), convert_code_to_subject(&class_code)));
                        minutes1 += 60;
                        difference -= 60;
                    }
                }
            } else {
                // The times seem possibly correct, so just use these times
                print_table.add_row(row![time_range.replace('-', " - "), class_code]);
                let start_time_12h = NaiveTime::parse_from_str(time_range.split('-').collect::<Vec<_>>()[0], "%H:%M")?.format("%I:%M&nbsp;%p");
                let finish_time_12h = NaiveTime::parse_from_str(time_range.split('-').collect::<Vec<_>>()[1], "%H:%M")?.format("%I:%M&nbsp;%p");
                let time_range_12h = format!("{} - {}", start_time_12h, finish_time_12h);
                timetable.push((time_range.replace('-', " - "), time_range_12h, class_code.clone(), convert_code_to_subject(&class_code)));
            }
        }
        timetables.push((date, timetable));
        print_table.printstd();
    }
    Ok(timetables)
}

pub fn scrape_timetables(class_name: &str) -> Result<Vec<Timetable>, failure::Error> {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()?;
    let (enrollment_no, username, password) = match class_name {
        "XI A₁" => ("<enrollment no>", "<id>", "<password>"),
        "XI A₂" => ("<enrollment no>", "<id>", "<password>"),
        "XI A₃" => ("<enrollment no>", "<id>", "<password>"),
        "XI A₄" => ("<enrollment no>", "<id>", "<password>"),
        "XI Z₁" => ("<enrollment no>", "<id>", "<password>"),
        "XI Z₂" => ("<enrollment no>", "<id>", "<password>"),
        "XI ESP" => ("<enrollment no>", "<id>", "<password>"),
        _ => return Err(failure::err_msg("Invalid page")),
    };
    let params = construct_payload(&client, username, password)?;

    // Get cookie
    client.post(URL)
        .form(&params)
        .send()?;

    // Sign in
    client.post(URL)
        .headers(construct_headers())
        .form(&params)
        .send()?;

    // Set enrollment number
    client.post("https://fiitjeelogin.com/Default.aspx")
        .headers(construct_headers())
        .form(&construct_payload_for_enrollment_no(&client, enrollment_no)?)
        .send()?;
    // Get the timetable page
    let mut res = client.post("https://fiitjeelogin.com/StudentTimeTable.aspx")
        .headers(construct_headers())
        .form(&construct_payload_for_timetable(&client)?)
        .send()?;
    let extracted_timetables = extract_timetable(&res.text()?, class_name == "XI ESP")?;
    let mut timetables = Vec::new();
    for date in extracted_timetables {
        timetables.push(Timetable {
            day: date.0,
            classes: date.1,
        });
    }
    Ok(timetables)
}
