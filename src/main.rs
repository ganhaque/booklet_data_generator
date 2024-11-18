use rusqlite::{params, Connection, Result};
use serde::Deserialize;
use serde_json::Value;
use std::fs;

#[derive(Debug, Deserialize)]
struct Lab {
  #[serde(rename = "type")]
  lab_type: String,
  begin: Value,
  end: Value,
  duration: Value,
  days: String,
  roomNumber: Option<String>,
  building: Option<String>,
  instructor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Course {
  available: i32,
  enrollmentCount: i32,
  capacity: i32,
  abbreviation: String,
  number: i32,
  #[serde(rename = "type")]
  course_type: Option<String>,
  section: i32,
  title: String,
  creditHour: String,
  begin: Value,
  end: Value,
  duration: Value,
  days: String,
  roomNumber: Option<String>,
  building: Option<String>,
  instructor: Option<String>,
  specialEnrollment: Option<String>,
  lab: Option<Lab>,
}

#[derive(Debug, Deserialize)]
struct Database {
  #[serde(flatten)]
  semesters: std::collections::HashMap<String, std::collections::HashMap<String, Vec<Course>>>,
}

fn parse_value_as_i32(value: &Value) -> Option<i32> {
  value.as_i64().map(|v| v as i32)
}

const DB_PATH: &str = "../cats.db";
const TABLES_PATH: &str = "./tables.sql";
const JSON_PATH: &str = "./booklet.json";

fn main() -> Result<()> {
  // Delete the database file if it exists
  if fs::metadata(DB_PATH).is_ok() {
    fs::remove_file(DB_PATH).expect("Failed to delete existing database file");
  }

  let conn = Connection::open(DB_PATH)?;

  // CREATE TABLES
  let sql_content = fs::read_to_string(TABLES_PATH).expect("Failed to read SQL file");
  conn.execute_batch(&sql_content).expect("Failed to execute SQL batch");

  // Read and parse JSON data
  let json_content = fs::read_to_string(JSON_PATH).expect("Failed to read JSON file");
  let database: Database = serde_json::from_str(&json_content).expect("Failed to parse JSON");

  // Insert data into the database
  insert_data(&conn, database)?;

  Ok(())
}

fn insert_data(conn: &Connection, database: Database) -> Result<()> {
  for (semester_title, departments) in database.semesters {
    // Skip some semesters
    if let Some(year_str) = semester_title.split_whitespace().last() {
      if let Ok(year) = year_str.parse::<u32>() {
        if year < 2018 {
          continue;
        }
      }
    }

    // Insert semester
    conn.execute(
      "INSERT OR IGNORE INTO semester (title) VALUES (?1)",
      params![semester_title],
    )?;

    for (department_title, courses) in departments {
      // Insert department
      let department_abbreviation = courses
        .get(0)
        .map(|course| course.abbreviation.clone())
        .unwrap_or_default();
      conn.execute(
        "INSERT OR IGNORE INTO department (title, abbreviation) VALUES (?1, ?2)",
        params![department_title, department_abbreviation],
      )?;

      for course in courses {
        // println!("processing: {:?}", course);
        // println!();

        // Insert course type if present
        if let Some(course_type) = &course.course_type {
          conn.execute(
            "INSERT OR IGNORE INTO course_type (type) VALUES (?1)",
            params![course_type],
          )?;
        }

        // Insert course template
        conn.execute(
          "INSERT OR IGNORE INTO course_template (department_title, number, title, credit_hour, type) VALUES (?1, ?2, ?3, ?4, ?5)",
          params![
            department_title,
            course.number,
            course.title,
            course.creditHour,
            course.course_type
          ],
        )?;

        // Insert schedule if times are present
        let schedule_key = if let (Value::Number(begin), Value::Number(end)) =
        (&course.begin, &course.end)
        {
          let begin_minutes = begin.as_i64().unwrap_or(0) as i32;
          let end_minutes = end.as_i64().unwrap_or(0) as i32;
          conn.execute(
            "INSERT OR IGNORE INTO schedule (time_begin, time_end, days_pattern) VALUES (?1, ?2, ?3)",
            params![begin_minutes, end_minutes, course.days],
          )?;
          Some((begin_minutes, end_minutes, course.days.clone()))
        } else {
          None
        };

        // Insert building and room if present
        if let (Some(building_name), Some(room_number)) =
        (&course.building, &course.roomNumber)
        {
          conn.execute(
            "INSERT OR IGNORE INTO building (name) VALUES (?1)",
            params![building_name],
          )?;
          conn.execute(
            "INSERT OR IGNORE INTO location (room_number, building_name) VALUES (?1, ?2)",
            params![room_number, building_name],
          )?;
        }

        // Insert instructor if present
        if let Some(instructor_name) = &course.instructor {
          conn.execute(
            "INSERT OR IGNORE INTO instructor (name) VALUES (?1)",
            params![instructor_name],
          )?;
        }

        // Insert special_enrollment if present
        if let Some(special_enrollment) = &course.specialEnrollment {
          conn.execute(
            "INSERT OR IGNORE INTO special_enrollment (type) VALUES (?1)",
            params![special_enrollment],
          )?;
        }

//         println!("INSERT OR IGNORE INTO course (
// semester_title, department_title, number, section,
// available, enrollment_count,
// instructor_name, room_number, building_name,
// time_begin, time_end, days_pattern, special_enrollment_type
// ) VALUES (
// {:?}, {:?}, {:?}, {:?},
// {:?}, {:?},
// {:?}, {:?}, {:?},
// {:?}, {:?}, {:?}, {:?}
// )",
//           semester_title,
//           department_title,
//           course.number,
//           course.section,
//           course.available,
//           course.enrollmentCount,
//           course.instructor,
//           course.roomNumber,
//           course.building,
//           schedule_key.as_ref().map(|(begin, _, _)| begin),
//           schedule_key.as_ref().map(|(_, end, _)| end),
//           schedule_key.as_ref().map(|(_, _, days)| days),
//           course.specialEnrollment,
//         );

        // Insert course
        conn.execute(
          "INSERT OR IGNORE INTO course (
semester_title, department_title, number, section,
available, enrollment_count,
instructor_name, room_number, building_name,
time_begin, time_end, days_pattern, special_enrollment_type
) VALUES (
?1, ?2, ?3, ?4,
?5, ?6,
?7, ?8, ?9,
?10, ?11, ?12, ?13
)",
          params![
            semester_title,
            department_title,
            course.number,
            course.section,
            course.available,
            course.enrollmentCount,
            course.instructor,
            course.roomNumber,
            course.building,
            schedule_key.as_ref().map(|(begin, _, _)| begin),
            schedule_key.as_ref().map(|(_, end, _)| end),
            schedule_key.as_ref().map(|(_, _, days)| days),
            course.specialEnrollment,
          ],
        )?;

        // Insert lab if present
        if let Some(lab) = &course.lab {
          // Insert course type if present
          // if let Some(course_type) = &lab.lab_type {
          // }
          conn.execute(
            "INSERT OR IGNORE INTO course_type (type) VALUES (?1)",
            params![lab.lab_type],
          )?;

          // Insert building and room if present
          if let (Some(building_name), Some(room_number)) =
          (&lab.building, &lab.roomNumber)
          {
            conn.execute(
              "INSERT OR IGNORE INTO building (name) VALUES (?1)",
              params![building_name],
            )?;
            conn.execute(
              "INSERT OR IGNORE INTO location (room_number, building_name) VALUES (?1, ?2)",
              params![room_number, building_name],
            )?;
          }

          // Insert instructor if present
          if let Some(instructor_name) = &lab.instructor {
            conn.execute(
              "INSERT OR IGNORE INTO instructor (name) VALUES (?1)",
              params![instructor_name],
            )?;
          }

          // Insert schedule if times are present
          let schedule_key = if let (Value::Number(begin), Value::Number(end)) =
          (&lab.begin, &lab.end)
          {
            let begin_minutes = begin.as_i64().unwrap_or(0) as i32;
            let end_minutes = end.as_i64().unwrap_or(0) as i32;
            conn.execute(
              "INSERT OR IGNORE INTO schedule (time_begin, time_end, days_pattern) VALUES (?1, ?2, ?3)",
              params![begin_minutes, end_minutes, lab.days],
            )?;
            Some((begin_minutes, end_minutes, lab.days.clone()))
          } else {
            None
          };

          conn.execute(
            "INSERT OR IGNORE INTO lab (
semester_title, department_title, course_number, section, type, instructor_name,
room_number, building_name, time_begin, time_end, days_pattern
) VALUES (
?1, ?2, ?3, ?4, ?5,
?6, ?7, ?8, ?9, ?10, ?11
)",
            params![
              semester_title,
              department_title,
              course.number,
              course.section,
              lab.lab_type,
              lab.instructor,
              lab.roomNumber,
              lab.building,
              schedule_key.as_ref().map(|(begin, _, _)| begin),
              schedule_key.as_ref().map(|(_, end, _)| end),
              schedule_key.as_ref().map(|(_, _, days)| days),
            ],
          )?;
        }
      }
    }
  }
  Ok(())
}
