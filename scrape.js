import puppeteer from "puppeteer";
// const fs = await import('fs');
import fs from 'fs/promises';
import he from 'he';
// const fs = require('fs/promises'); // Import the filesystem module
// import { parseData } from "./parse.js";

async function scrape() {
  // const parser = new DOMParser();
  console.log("[START SCRAPE]");
  const browser = await puppeteer.launch({ headless: "new"});
  const page = await browser.newPage();

  // navigate to the target website
  await page.goto('http://appl101.lsu.edu/booklet2.nsf/Selector2?OpenForm');

  // initialize and populate data
  let database = {};
  let semesters = await page.evaluate(() => {
    const select_element = document.querySelector('select[name="SemesterDesc"]');
    const option_elements = select_element ? Array.from(select_element.querySelectorAll('option')) : [];
    const options = option_elements.map(option => option.textContent.trim());
    return options;
  });
  let departments = await page.evaluate(() => {
    const select_element = document.querySelector('select[name="Department"]');
    const option_elements = select_element ? Array.from(select_element.querySelectorAll('option')) : [];
    const options = option_elements.map(option => option.textContent.trim());
    return options;
  });

  // filter out some semesters and departments
  semesters = semesters.filter(str =>
    !str.toLowerCase().includes("module") &&
      !str.toLowerCase().includes("intersession") &&
      !str.toLowerCase().includes("summer")
  );
  departments = departments.filter(str =>
    !str.startsWith("*") &&
      !str.startsWith("+")
  );

  for (const semester_title of semesters) {
    // console.log("[SEMESTER]", semester_title);
    // Optional: Skip some older semesters
    const regex = /^(Spring|Fall) (\d{4})$/;
    let match = semester_title.match(regex);
    if (match) {
      let year = parseInt(match[2], 10);
      if (year < 2024) {
        continue;
      }
    }

    // intialize
    if (!database[semester_title]) {
      database[semester_title] = {};
    }

    for (const department_title of departments) {
      // console.log("[DEPARTMENT]", department_title);
      // intialize
      if (!database[semester_title][department_title]) {
        database[semester_title][department_title] = [];
      }

      // navigate
      await page.select('select[name="SemesterDesc"]', semester_title);
      await page.select('select[name="Department"]', department_title);
      await page.waitForSelector('input[type="submit"][value="Display Courses"]');
      await page.click('input[type="submit"][value="Display Courses"]');

      // wait for new page
      const target = await browser.waitForTarget(target => target.opener() === page.target());
      const new_page = await target.page();

      // 404
      if (new_page.url() == "https://appl101.lsu.edu/booklet2.nsf/NoCourseDept?readform") {
        console.log("\x1b[33m%s\x1b[0m", "[404]", semester_title, department_title);
      }
      else {
        const html_content = await new_page.content();

        let pre_element;
        const match = html_content.match(/<pre>([\s\S]*?)<\/pre>/);
        if (!match) {
          console.log("No <pre> element found in the HTML.");
        } else {
          pre_element = match[1].trim();
          // replace amp for utf-8 compliance
          // pre_element = pre_element.replace(/&amp;/g, '&');
          // const decoded_pre_element = pre_element;
          const decoded_pre_element = he.decode(pre_element);
          // console.log(decoded_pre_element);

          let courses = await parse(decoded_pre_element)
            .then((result) => {
              console.log('\x1b[32m%s\x1b[0m', "[SUCCESS]", semester_title, department_title);
              return result;
            })
            .catch(() => {
              console.log('\x1b[31m%s\x1b[0m', "[PARSE FAILED]", semester_title, department_title);
              return null;
            })
          ;

          if (courses) {
            database[semester_title][department_title] = courses;
          }
        }
      }

      // close page to go back to original
      await new_page.close();
    }
  }

  try {
    await fs.writeFile('./booklet.json', JSON.stringify(database), 'utf8');
    console.log('booklet.json has been written');
  } catch (err) {
    console.error('Error writing file:', err);
  }

  await browser.close();
}

async function parse(data) {
  const lines = data.split('\n');
  let courses = [];

  const ranges = [
    { name: "available", start: 0, end: 4 }, // special cases: `(F)` or ``
    { name: "enrollment", start: 5, end: 10 }, // special cases: ``
    { name: "abbreviation", start: 11, end: 15 }, // special cases: ABBR does not match departments
    { name: "course_number", start: 16, end: 20 },
    { name: "course_type", start: 21, end: 25 }, // special cases: ``
    { name: "section", start: 27, end: 31 },
    { name: "course_title", start: 32, end: 53 },
    { name: "credit_hour", start: 55, end: 59 }, // special cases: ``, 3.0, 1-12
    { name: "time", start: 60, end: 70 }, // special cases: ``
    { name: "days", start: 72, end: 77 }, // special cases: ``
    { name: "room", start: 79, end: 83 }, // special cases: ``
    { name: "building", start: 84, end: 99 }, // special cases: ``
    { name: "special_enrollment", start: 100, end: 116 }, // special cases: `` or multiple
    { name: "instructor", start: 117, end: -1 } // end is end of line
  ];

  let previous_course = null;

  for (let i = 3; i < lines.length; i++) {
    let line = lines[i];

    // skip invalid lines
    if (line.trim().startsWith("*")) {
      continue;
    }
    else if (line.trim().startsWith("Spring") || line.trim().startsWith("Fall")) {
      continue;
    }
    else if (line.trim().startsWith("SESSION")) {
      continue;
    }

    let course = {};

    // partition the line
    ranges.forEach(range => {
      const { name, start, end } = range;
      // adjust the `end` index if it exceeds the line length
      const adjusted_end = (end === -1 || end > line.length) ? line.length : end;

      const value = line.slice(start, adjusted_end);
      const trimmed_value = value.trim();

      // only add non-blank values
      if (trimmed_value.length > 0) {
        course[name] = trimmed_value;
      }
    });

    // check if this is an extension
    if (
      course["course_type"] &&
        course["course_type"].length == 3 &&
        !course["abbreviation"] &&
        !course["course_number"] &&
        !course["section"]
    ) {
      previous_course.extension = course;
    }
    else {
      // add to the list if it has any valid data
      if (Object.keys(course).length > 0) {
        // if these fields are not null
        if (course.section && course.abbreviation && course.course_number) {
          courses.push(course);
          previous_course = course;
        }
      }
    }
  }

  // merge similar courses since there can be multiple entries
  // group by unique course identifiers
  const courseMap = new Map();

  courses.forEach(course => {
    const key = `${course.abbreviation || ''}-${course.course_number || ''}-${course.section || ''}`;

    if (courseMap.has(key)) {
      const existingCourse = courseMap.get(key);

      // take first non-null/blank value
      Object.keys(course).forEach(field => {
        if (!existingCourse[field] || existingCourse[field].length === 0) {
          existingCourse[field] = course[field];
        }
      });
    } else {
      courseMap.set(key, { ...course });
    }
  });

  // Convert the map back to an array
  courses = Array.from(courseMap.values());

  return courses;
}

async function test_parse() {
  try {
    const data = await fs.readFile('./test_parse.txt', 'utf8');
    const parsed_data = await parse(data);

    // Write the parsed data to a new JSON file
    await fs.writeFile('./test_parse.json', JSON.stringify(parsed_data), 'utf8');
    console.log('test_parse.json file written');
  } catch (err) {
    console.error('error:', err);
  }
}

// test_parse();
scrape();
