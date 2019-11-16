use super::{
    TimetableList, Timetable, Logs, PushSubscription, class_url_to_name, webscrape::*
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
            times_and_classes.push(format!("{}={}={}", row.0, row.1, row.2));
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
    // Variable to store the timetables we want
    let mut timetables = Vec::new();
    // Loop over all the timetables of this class in the database
    for timetable in &conn.query("SELECT day, timetable FROM timetables WHERE class_name = $1 ORDER BY id", &[&class_name])? {
        // Check whether the day this timetable is for is after the start of this week
        let now = Local::today().naive_local();
        let day = NaiveDate::parse_from_str(&timetable.get::<_, String>(0), "%A, %B %e, %Y")?;

        let start_of_the_week = now - chrono::Duration::days(now.weekday().num_days_from_sunday() as i64);

        if start_of_the_week <= day {
            // It is after the start of the week, so add it to the timetables vector
            let mut classes = Vec::new();
            for row in timetable.get::<_, String>(1).split("|") {
                let time_and_class = row.split("=").collect::<Vec<_>>();
                classes.push((time_and_class[0].to_string(), time_and_class[1].to_string(), time_and_class[2].to_string()));
            }
            timetables.push(Timetable {
                day: timetable.get(0),
                classes,
            });
        }
    }

    // If no timetables after the start of this week were found, scrape them from fiitjeelogin
    if timetables.len() == 0 {
        timetables.append(&mut scrape_timetables(&class_name)?);
        log_action(conn, Action::AutoFetched)?;
        // Add the timetables to the database so we don't have to scrape them next time
        insert_timetables(conn, TimetableList {
            class_name: class_name.clone(),
            timetables: timetables.clone(),
        }, false)?;
    }

    Ok(TimetableList {
        class_name,
        timetables,
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
    }, true) {
        return e.to_string();
    }
    String::from("success")
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
