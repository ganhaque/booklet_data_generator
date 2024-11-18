CREATE TABLE semester (
  title TEXT PRIMARY KEY
);

CREATE TABLE department (
  title TEXT PRIMARY KEY,
  abbreviation TEXT -- nullable
);

CREATE TABLE course_type (
  type TEXT PRIMARY KEY -- e.g., "LAB", "REC", etc.
);

CREATE TABLE course_template (
  department_title TEXT NOT NULL,
  number INTEGER NOT NULL,
  title TEXT NOT NULL,
  credit_hour TEXT, -- e.g., "3.0", "4.0", "1-3", etc.
  type TEXT, -- nullable
  FOREIGN KEY (department_title) REFERENCES department(title),
  FOREIGN KEY (type) REFERENCES course_type(type),
  PRIMARY KEY (department_title, number)
);

CREATE TABLE schedule (
  time_begin INTEGER NOT NULL, -- in minutes from midnight
  time_end INTEGER NOT NULL,   -- in minutes from midnight
  days_pattern TEXT NOT NULL,  -- e.g., "MWF", "TR"
  PRIMARY KEY (time_begin, time_end, days_pattern)
);

CREATE TABLE building (
  name TEXT PRIMARY KEY
);

CREATE TABLE location (
  room_number TEXT NOT NULL,
  building_name TEXT NOT NULL,
  FOREIGN KEY (building_name) REFERENCES building(name),
  PRIMARY KEY (room_number, building_name)
);

CREATE TABLE special_enrollment (
  type TEXT PRIMARY KEY -- e.g., "100% WEB BASED"
);

CREATE TABLE instructor (
  name TEXT PRIMARY KEY
);

CREATE TABLE course (
  semester_title TEXT NOT NULL,
  department_title TEXT NOT NULL,
  number INTEGER NOT NULL,
  section INTEGER NOT NULL,
  available INTEGER NOT NULL,
  enrollment_count INTEGER NOT NULL,
  instructor_name TEXT, -- nullable
  room_number TEXT, -- nullable
  building_name TEXT, -- nullable
  time_begin INTEGER, -- nullable
  time_end INTEGER, -- nullable
  days_pattern TEXT, -- nullable
  special_enrollment_type TEXT, -- nullable
  FOREIGN KEY (semester_title) REFERENCES semester(title),
  FOREIGN KEY (department_title, number) REFERENCES course_template(department_title, number),
  FOREIGN KEY (room_number, building_name) REFERENCES location(room_number, building_name),
  FOREIGN KEY (time_begin, time_end, days_pattern) REFERENCES schedule(time_begin, time_end, days_pattern),
  FOREIGN KEY (special_enrollment_type) REFERENCES special_enrollment(type),
  FOREIGN KEY (instructor_name) REFERENCES instructor(name),
  PRIMARY KEY (semester_title, department_title, number, section)
);

CREATE TABLE lab (
  semester_title TEXT NOT NULL,
  department_title TEXT NOT NULL,
  course_number INTEGER NOT NULL,
  section INTEGER NOT NULL,
  type TEXT NOT NULL,
  instructor_name TEXT, -- nullable
  room_number TEXT, -- nullable
  building_name TEXT, -- nullable
  time_begin INTEGER, -- nullable
  time_end INTEGER, -- nullable
  days_pattern TEXT, -- nullable
  FOREIGN KEY (semester_title, department_title, course_number, section) REFERENCES course(semester_title, department_title, number, section),
  FOREIGN KEY (type) REFERENCES course_type(type),
  FOREIGN KEY (room_number, building_name) REFERENCES location(room_number, building_name),
  FOREIGN KEY (time_begin, time_end, days_pattern) REFERENCES schedule(time_begin, time_end, days_pattern),
  FOREIGN KEY (instructor_name) REFERENCES instructor(name),
  PRIMARY KEY (semester_title, department_title, course_number, section)
);
