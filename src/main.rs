mod helper;

use helper::parse_time_string;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct Extension {
  course_type: Option<String>,
  time: Option<String>,
  days: Option<String>,
  room: Option<String>,
  building: Option<String>,
  instructor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Course {
  available: Option<String>,
  enrollment: Option<String>,
  abbreviation: Option<String>,
  course_number: Option<String>,
  course_type: Option<String>,
  section: Option<String>,
  course_title: Option<String>,
  credit_hour: Option<String>,
  time: Option<String>,
  days: Option<String>,
  room: Option<String>,
  building: Option<String>,
  special_enrollment: Option<String>,
  instructor: Option<String>,
  extension: Option<Extension>,
}

#[derive(Debug, Deserialize)]
struct Database {
  #[serde(flatten)]
  semesters: std::collections::HashMap<String, std::collections::HashMap<String, Vec<Course>>>,
}

const DB_PATH: &str = "../booklet.db";
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

  initialize_semester_table(&conn, &database);
  initialize_department_table(&conn, &database);
  initialize_course_table(&conn, database);

  Ok(())
}

fn initialize_semester_table(conn: &Connection, database: &Database) {
  for (semester_title, _) in &database.semesters {
    let _ = conn.execute(
      "INSERT INTO semester (semester_title) VALUES (?1)",
      params![semester_title],
    );
  }
}

fn initialize_department_table(conn: &Connection, database: &Database) {
  for (_semester_title, departments) in &database.semesters {
    for (department_title, courses) in departments {
      let department_abbreviation = courses
        .get(0)
        .map(|course| course.abbreviation.clone())
        .unwrap_or_default();
      let _ = conn.execute(
        "INSERT OR IGNORE INTO department (department_title, abbreviation) VALUES (?1, ?2)",
        params![department_title, department_abbreviation],
      );
    }
  }
}

fn initialize_course_table(conn: &Connection, database: Database) {
  for (semester_title, departments) in database.semesters {
    for (department_title, courses) in departments {
      println!("\x1b[32m[SUCCESS]\x1b[0m {}, {}", semester_title, department_title);
      for course in courses {
        // println!("course: {:?}", course);
        let available = course.available.map(|s| s.parse::<i32>().unwrap_or(0));
        let enrollment = course.enrollment.map(|s| s.parse::<i32>().unwrap_or(0));
        initialize_course_type_table(conn, &course.course_type);
        initialize_credit_hour_table(conn, &course.credit_hour);
        // println!("section: {:?}", course.section);
        let section = match &course.section {
          Some(s) => s.parse::<i32>().expect("invalid section"),
          None => panic!("section is null"),
        };
        let course_number = initialize_course_template_table(
          conn,
          course.course_number,
          &course.course_title,
          &course.credit_hour,
          &course.course_type,
          &department_title
        );
        let (time_begin, time_end, day_pattern) = match initialize_time_slot_table(
          conn,
          &course.time,
          &course.days,
        ) {
          Some((begin, end, day_pattern)) => (Some(begin), Some(end), Some(day_pattern)),
          None => (None, None, None),
        };
        initialize_location_table(
          conn,
          &course.room,
          &course.building,
        );
        initialize_special_enrollment_table(
          conn,
          &course.special_enrollment
        );
        initialize_instructor_table(
          conn,
          &course.instructor
        );
        let course_extension_id = initialize_course_extension_table(
          conn,
          &course.extension,
        );

        conn.execute(r#"
          INSERT INTO course (
            semester_title,
            department_title,
            available,
            enrollment,
            course_number,
            section,
            room_number,
            building_name,
            time_begin,
            time_end,
            day_pattern,
            special_enrollment,
            instructor_name,
            course_extension_id
          ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)"#,
          params![
            semester_title,
            department_title,
            available,
            enrollment,
            course_number,
            section,
            course.room,
            course.building,
            time_begin,
            time_end,
            day_pattern,
            course.special_enrollment,
            course.instructor,
            course_extension_id,
          ],
        ).expect("err");

      }
    }
  }
}

fn initialize_course_type_table(
  conn: &Connection,
  course_type: &Option<String>,
) {
  if let Some(course_type) = &course_type {
    let _ = conn.execute(
      "INSERT OR IGNORE INTO course_type (course_type) VALUES (?1)",
      params![course_type],
    );
  }
}

fn initialize_credit_hour_table(
  conn: &Connection,
  credit_hour: &Option<String>,
) {
  let Some(credit_hour) = &credit_hour else {return};
  let _ = conn.execute(
    "INSERT OR IGNORE INTO credit_hour (credit_hour) VALUES (?1)",
    params![credit_hour],
  );
}

fn initialize_course_template_table(
  conn: &Connection,
  course_number: Option<String>,
  course_title: &Option<String>,
  credit_hour: &Option<String>,
  course_type: &Option<String>,
  department_title: &String,
) -> i32{
  // println!("course_number: {:?}", course_number);
  let course_number = &course_number.unwrap().parse::<i32>().expect("invalid course_number");

  let _ = conn.execute(
    r#"INSERT OR IGNORE INTO course_template (
      department_title,
      course_number,
      course_title,
      credit_hour,
      course_type
    ) VALUES (?1, ?2, ?3, ?4, ?5)"#,
    params![
      department_title,
      course_number,
      course_title,
      credit_hour,
      course_type,
    ],
  );

  return *course_number;
}

fn initialize_time_slot_table(
  conn: &Connection,
  time: &Option<String>,
  days: &Option<String>,
) -> Option<(i32, i32, String)> {
  let Some(time) = &time else {return None};
  let Some(day_pattern) = &days else {return None};

  let Ok((begin, end)) = parse_time_string(time) else {return None};
  let _ = conn.execute(
    "INSERT OR IGNORE INTO day_pattern (day_pattern) VALUES (?1)",
    params![day_pattern],
  );
  let _ = conn.execute(
    "INSERT OR IGNORE INTO time_slot (time_begin, time_end, day_pattern) VALUES (?1, ?2, ?3)",
    params![begin, end, day_pattern],
  );
  return Some((begin, end, day_pattern.to_string()));
}

fn initialize_location_table(
  conn: &Connection,
  room: &Option<String>,
  building: &Option<String>,
) {
  let Some(room_number) = &room else {return};
  let Some(building) = &building else {return};
  let _ = conn.execute(
    "INSERT OR IGNORE INTO building (building_name) VALUES (?1)",
    params![building],
  );
  let _ = conn.execute(
    "INSERT OR IGNORE INTO location (room_number, building_name) VALUES (?1, ?2)",
    params![room_number, building],
  );
}

fn initialize_special_enrollment_table(
  conn: &Connection,
  special_enrollment: &Option<String>,
) {
  let Some(special_enrollment) = &special_enrollment else {return};
  let _ = conn.execute(
    "INSERT OR IGNORE INTO special_enrollment (special_enrollment) VALUES (?1)",
    params![special_enrollment],
  );
}

fn initialize_instructor_table(
  conn: &Connection,
  instructor: &Option<String>
) {
  let Some(instructor) = &instructor else {return};
  let _ = conn.execute(
    "INSERT OR IGNORE INTO instructor (instructor_name) VALUES (?1)",
    params![instructor],
  );
}

fn initialize_course_extension_table(
  conn: &Connection,
  extension: &Option<Extension>
) -> Option<i64>{
  let Some(extension) = &extension else {return None};

  // initialize_extension_type
  let _ = conn.execute(
    "INSERT OR IGNORE INTO extension_type (extension_type) VALUES (?1)",
    params![extension.course_type],
  );

  initialize_location_table(
    conn,
    &extension.room,
    &extension.building,
  );
  let (time_begin, time_end, day_pattern) = match initialize_time_slot_table(
    conn,
    &extension.time,
    &extension.days,
  ) {
    Some((begin, end, day_pattern)) => (Some(begin), Some(end), Some(day_pattern)),
    None => (None, None, None),
  };
  initialize_instructor_table(
    conn,
    &extension.instructor
  );

  conn.execute(r#"
    INSERT INTO course_extension (
      extension_type,
      room_number,
      building_name,
      time_begin,
      time_end,
      day_pattern,
      instructor_name
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
    params![
      extension.course_type,
      extension.room,
      extension.building,
      time_begin,
      time_end,
      day_pattern,
      extension.instructor,
    ],
  ).expect("err");

  // return auto-increment id
  Some(conn.last_insert_rowid())
}

