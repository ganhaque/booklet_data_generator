CREATE TABLE semester (
  semester_title TEXT PRIMARY KEY
);

CREATE TABLE department (
  department_title TEXT PRIMARY KEY,
  abbreviation TEXT -- nullable
);

CREATE TABLE course_type (
  course_type TEXT PRIMARY KEY -- e.g., "LAB", "REC", etc.
);

CREATE TABLE credit_hour (
  credit_hour TEXT PRIMARY KEY -- e.g., "3.0", "4.0", "1-3", etc.
);

CREATE TABLE course_template (
  department_title TEXT NOT NULL,
  course_number INTEGER NOT NULL,
  course_title TEXT NOT NULL,
  credit_hour TEXT, -- nullable
  course_type TEXT, -- nullable
  FOREIGN KEY (department_title) REFERENCES department(department_title),
  FOREIGN KEY (course_type) REFERENCES course_type(course_type) ON DELETE SET NULL,
  FOREIGN KEY (credit_hour) REFERENCES credit_hour(credit_hour) ON DELETE SET NULL,
  PRIMARY KEY (department_title, course_number)
);

CREATE TABLE day_pattern (
  day_pattern TEXT PRIMARY KEY -- e.g., "MWF", "TR"
);

CREATE TABLE time_slot (
  time_begin INTEGER NOT NULL, -- in minutes from midnight
  time_end INTEGER NOT NULL,   -- in minutes from midnight
  PRIMARY KEY (time_begin, time_end)
);

CREATE TABLE building (
  building_name TEXT PRIMARY KEY
);

CREATE TABLE location (
  room_number TEXT NOT NULL,
  building_name TEXT NOT NULL,
  FOREIGN KEY (building_name) REFERENCES building(building_name),
  PRIMARY KEY (room_number, building_name)
);

CREATE TABLE special_enrollment (
  special_enrollment TEXT PRIMARY KEY -- e.g., "100% WEB BASED"
);

CREATE TABLE instructor (
  instructor_name TEXT PRIMARY KEY
);

CREATE TABLE course (
  course_id INTEGER PRIMARY KEY AUTOINCREMENT,
  semester_title TEXT NOT NULL,
  department_title TEXT NOT NULL,
  available INTEGER, -- nullable
  enrollment INTEGER, -- nullable
  course_number INTEGER NOT NULL,
  section INTEGER NOT NULL,
  room_number TEXT, -- nullable
  building_name TEXT, -- nullable
  time_begin INTEGER, -- nullable
  time_end INTEGER, -- nullable
  day_pattern TEXT, -- nullable
  special_enrollment TEXT, -- nullable
  instructor_name TEXT, -- nullable
  FOREIGN KEY (semester_title) REFERENCES semester(semester_title),
  FOREIGN KEY (department_title, course_number) REFERENCES course_template(department_title, course_number),
  FOREIGN KEY (room_number, building_name) REFERENCES location(room_number, building_name) ON DELETE SET NULL,
  FOREIGN KEY (time_begin, time_end) REFERENCES time_slot(time_begin, time_end) ON DELETE SET NULL,
  FOREIGN KEY (day_pattern) REFERENCES day_pattern(day_pattern) ON DELETE SET NULL,
  FOREIGN KEY (special_enrollment) REFERENCES special_enrollment(special_enrollment) ON DELETE SET NULL,
  FOREIGN KEY (instructor_name) REFERENCES instructor(instructor_name) ON DELETE SET NULL
);

-- CREATE TABLE user (
--   username TEXT PRIMARY KEY NOT NULL,
--   password TEXT NOT NULL
-- );
--
-- CREATE TABLE schedule (
--   schedule_id INTEGER PRIMARY KEY AUTOINCREMENT,
--   schedule_name TEXT NOT NULL,
--   username TEXT NOT NULL,
--   semester_title TEXT NOT NULL,
--   FOREIGN KEY (username) REFERENCES user(username),
--   FOREIGN KEY (semester_title) REFERENCES semester(semester_title)
-- );
--
-- CREATE TABLE schedule_course (
--   schedule_id INTEGER NOT NULL,
--   course_id INTEGER NOT NULL,
--   FOREIGN KEY (schedule_id) REFERENCES schedule(schedule_id),
--   FOREIGN KEY (course_id) REFERENCES course(course_id),
--   PRIMARY KEY (schedule_id, course_id)
-- );
