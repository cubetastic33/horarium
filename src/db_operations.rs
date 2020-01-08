use super::{
    TimetableList, Timetable, Analysis, Logs, PushSubscription, class_url_to_name, webscrape::*
};
use chrono::prelude::*;
use postgres::Connection;

enum Action {
    AutoFetched,
    ForciblyFetched,
    InsertedTimetables,
    DeletedTimetables,
}

// Function to check if a timetable already exists
fn already_exists(conn: &Connection, class_name: &str, day: &str) -> Result<bool, failure::Error> {
    Ok(!conn.query("SELECT * FROM timetables WHERE class_name = $1 AND day = $2", &[&class_name, &day])?.is_empty())
}

/*
CREATE TABLE timetables (
id serial PRIMARY KEY,
class_name VARCHAR (6) NOT NULL,
day VARCHAR (30) NOT NULL,
timetable VARCHAR NOT NULL
)
*/

/*
CREATE TABLE actions_log (
id serial PRIMARY KEY,
action VARCHAR (30) NOT NULL,
time TIMESTAMPTZ NOT NULL DEFAULT NOW()
)

SET timezone = 'Asia/Kolkata';
*/

/*
CREATE TABLE notification_subscriptions (
id serial PRIMARY KEY,
push_subscription VARCHAR NOT NULL
)
*/

// Function to log actions in the database
fn log_action(conn: &Connection, action: Action) -> Result<(), failure::Error> {
    conn.execute("INSERT INTO actions_log VALUES (DEFAULT, $1, DEFAULT)", &[&match action {
        Action::AutoFetched => "Auto Fetched",
        Action::ForciblyFetched => "Forcibly Fetched",
        Action::InsertedTimetables => "Inserted Timetables",
        Action::DeletedTimetables => "Deleted Timetables",
    }])?;
    Ok(())
}

// Function to insert a new timetable into the database
fn insert_timetables(conn: &Connection, timetable_list: TimetableList, force_insert: bool) -> Result<(), failure::Error> {
    let mut inserted_something = false;
    let mut deleted_something = false;
    // Loop over all the timetables
    for timetable in timetable_list.timetables {
        if force_insert {
            // If we need to forcibly insert, delete any existing entry for this class for this day
            conn.execute("DELETE FROM timetables WHERE class_name = $1 AND day = $2", &[&timetable_list.class_name, &timetable.day])?;
            deleted_something = true;
        } else if already_exists(conn, &timetable_list.class_name, &timetable.day)? {
            // Skip if there's already a timetable for this class for this day
            continue;
        }
        let mut times_and_classes = Vec::new();
        for row in timetable.classes {
            times_and_classes.push(format!("{}={}={}={}", row.0, row.1, row.2, row.3));
        }
        // Insert the timetable into the database
        conn.query("INSERT INTO timetables VALUES (
                DEFAULT, $1, $2, $3
            )", &[
                &timetable_list.class_name,
                &timetable.day,
                &times_and_classes.join("|")
            ])?;
        inserted_something = true;
    }
    if deleted_something {
        log_action(conn, Action::DeletedTimetables)?;
    }
    if inserted_something {
        log_action(conn, Action::InsertedTimetables)?;
    }
    Ok(())
}

// Function to get the timetables
pub fn get_timetables(conn: &Connection, class_url: String) -> Result<TimetableList, failure::Error> {
    let class_name = class_url_to_name(&class_url);
    // Variable to store timetables from after the start of this week
    let mut timetables = Vec::new();
    // Variable to store this week's timetables
    let mut this_week_timetables = Vec::new();
    // Variable to store last week's timetables
    let mut last_week_timetables = Vec::new();
    // Loop over all the timetables of this class in the database
    for timetable in &conn.query("SELECT day, timetable FROM timetables WHERE class_name = $1 ORDER BY id", &[&class_name])? {
        // Check whether the day this timetable is for is after the start of this week
        let now = Local::today().naive_local();
        let day = NaiveDate::parse_from_str(&timetable.get::<_, String>(0), "%A, %B %e, %Y")?;

        let start_of_last_week = now - chrono::Duration::days(now.weekday().num_days_from_sunday() as i64 + 7);
        let start_of_this_week = now - chrono::Duration::days(now.weekday().num_days_from_sunday() as i64);
        let end_of_this_week = start_of_this_week + chrono::Duration::weeks(1);

        if start_of_this_week <= day {
            // It is after the start of this week, so add it to the timetables vector
            let mut classes = Vec::new();
            for row in timetable.get::<_, String>(1).split("|") {
                let time_and_class = row.split("=").collect::<Vec<_>>();
                classes.push((time_and_class[0].to_string(), time_and_class[1].to_string(), time_and_class[2].to_string(), time_and_class[3].to_string()));
            }
            timetables.push(Timetable {
                day: timetable.get(0),
                classes: classes.clone(),
            });
            if day <= end_of_this_week {
                this_week_timetables.push(Timetable {
                    day: timetable.get(0),
                    classes,
                });
            }
        } else if start_of_last_week <= day {
            // It is only after the start of last week, not this one, so add it
            // to the vector with last week's timetables
            let mut classes = Vec::new();
            for row in timetable.get::<_, String>(1).split("|") {
                let time_and_class = row.split("=").collect::<Vec<_>>();
                classes.push((time_and_class[0].to_string(), time_and_class[1].to_string(), time_and_class[2].to_string(), time_and_class[3].to_string()));
            }
            last_week_timetables.push(Timetable {
                day: timetable.get(0),
                classes,
            });
        }
    }

    // If no timetables after the start of this week were found, scrape them from fiitjeelogin
    if timetables.len() == 0 {
        let mut temp = scrape_timetables(&class_name)?;
        timetables.append(&mut temp.clone());
        this_week_timetables.append(&mut temp);
        log_action(conn, Action::AutoFetched)?;
        // Add the timetables to the database so we don't have to scrape them next time
        insert_timetables(conn, TimetableList {
            class_name: class_name.clone(),
            timetables: timetables.clone(),
            analyses: None,
        }, false)?;
    }

    let this_week_analyses = generate_analyses(&this_week_timetables)?;
    println!("\n\nlast week\n\n");
    let last_week_analyses = generate_analyses(&last_week_timetables)?;

    Ok(TimetableList {
        class_name,
        timetables,
        analyses: Some([this_week_analyses, last_week_analyses]),
    })
}

// Function to forcibly fetch timetables from fiitjeelogin
pub fn forcibly_fetch(conn: &Connection, class_url: String) -> String {
    let class_name = class_url_to_name(&class_url);
    let timetables = match scrape_timetables(&class_name) {
        Ok(x) => x,
        Err(e) => return e.to_string(),
    };
    if let Err(e) = log_action(conn, Action::ForciblyFetched) {
        return e.to_string();
    }
    if let Err(e) = insert_timetables(conn, TimetableList {
        class_name,
        timetables,
        analyses: None,
    }, true) {
        return e.to_string();
    }
    String::from("success")
}

// Function to generate analyses from timetables
fn generate_analyses(timetables: &Vec<Timetable>) -> Result<Vec<Analysis>, failure::Error> {
    let mut analyses = Vec::new();
    // Only include recognized subjects
    let mut math = 0;
    let mut botany = 0;
    let mut zoology = 0;
    let mut chemistry = 0;
    let mut physics = 0;
    let mut english = 0;
    let mut computer_science = 0;
    let mut bio_or_cs = 0;
    let mut games = 0;
    // Loop over each day
    for timetable in timetables {
        // Loop over each class
        for class in &timetable.classes {
            println!("\n{}", class.3);
            *match class.3.as_ref() {
                "Math" => &mut math,
                "Botany" => &mut botany,
                "Zoology" => &mut zoology,
                "Chemistry" => &mut chemistry,
                "Physics" => &mut physics,
                "English" => &mut english,
                "Computer Science" => &mut computer_science,
                "Bio / CS" => &mut bio_or_cs,
                "Games" => &mut games,
                _ => continue,
            } += {
                // The x.trim() is for backwards compatability with previous weeks
                let times: Vec<_> = class.0.split('-').map(|x| x.trim()).collect();
                (NaiveTime::parse_from_str(times[1], "%H:%M")? - NaiveTime::parse_from_str(times[0], "%H:%M")?).num_minutes()
            };
        }
    }
    let subjects_list = [
        ("Math", math),
        ("Botany", botany),
        ("Zoology", zoology),
        ("Chem", chemistry),
        ("Phy", physics),
        ("Eng", english),
        ("CS", computer_science),
        ("Bio / CS", bio_or_cs),
        ("Games", games),
    ];
    // subjects_list.sort_unstable_by(|a, b| b.1.cmp(&a.1));
    for subject in &subjects_list {
        if subject.1 > 0 {
            /*let time = chrono::Duration::minutes(subject.1);
            let hours = time.num_hours();
            let minutes = time.num_minutes() - hours * 60;
            let aggregate_time = if hours == 1 {
                if minutes > 0 {
                    format!("1 hour and {} minutes", minutes)
                } else {
                    String::from("1 hour")
                }
            } else {
                if minutes > 0 {
                    format!("{} hours and {} minutes", hours, minutes)
                } else {
                    format!("{} hours", hours)
                }
            };*/
            analyses.push(Analysis {
                name: subject.0.to_string(),
                aggregate_time: subject.1,
            });
        }
    }
    Ok(analyses)
}

// Function to get the logs from the database
pub fn get_logs(conn: &Connection) -> Result<Logs, failure::Error> {
    let mut logs = Logs::default();
    for log in &conn.query("SELECT * FROM actions_log ORDER BY id DESC", &[])? {
        let action_as_class = String::from(match log.get::<_, String>(1).as_ref() {
            "Auto Fetched" => "autoFetched",
            "Forcibly Fetched" => "forciblyFetched",
            "Inserted Timetables" => "insertedTimetables",
            "Deleted Timetables" => "deletedTimetables",
            x => x,
        });
        let action_time: DateTime<Local> = log.get(2);
        logs.logs.push((action_as_class, log.get(1), action_time.format("1%F %T").to_string()));
    }
    Ok(logs)
}

// Function to add a notification subscription to the database
pub fn add_notification_subscription(conn: &Connection, push_subscription: PushSubscription) -> Result<String, failure::Error> {
    conn.execute("INSERT INTO notification_subscriptions VALUES (
        DEFAULT, $1
    )", &[&serde_json::to_string(&push_subscription)?])?;
    Ok(String::from("success"))
}
