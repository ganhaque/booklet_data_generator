struct Time {
  hour: i32,
  minute: i32,
}

fn parse_time(input: &str) -> Result<Time, &'static str> {
  // println!("parsing time: {}", input);
  if input.len() != 3 && input.len() != 4 {
    return Err("Invalid input");
  }

  let (hour, minute) = if input.len() == 3 {
    let hour = input[0..1].parse::<i32>().map_err(|_| "Invalid time")?;
    let minute = input[1..].parse::<i32>().map_err(|_| "Invalid time")?;
    (hour, minute)
  } else {
    let hour = input[0..2].parse::<i32>().map_err(|_| "Invalid time")?;
    let minute = input[2..].parse::<i32>().map_err(|_| "Invalid time")?;
    (hour, minute)
  };

  if hour > 23 || minute > 59 {
    return Err("Invalid time");
  }

  Ok(Time {
    hour,
    minute,
  })
}

pub fn parse_time_string(time: &String) -> Result<(i32, i32), &'static str> {
  let mut is_night = false;
  let time_string = if time.ends_with('N') {
    is_night = true;
    time[0..time.len() - 1].to_string()
  } else {
    time.clone()
  };

  // println!("parsing time string: {}", time_string);

  let parts: Vec<&str> = time_string.split('-').collect();
  if parts.len() != 2 {return Err("")}
  let Ok(mut begin) = parse_time(parts[0]) else {return Err("")};
  let Ok(mut end) = parse_time(parts[1]) else {return Err("")};

  if is_night {
    begin.hour += 12;
    end.hour += 12;
  }
  else {
    if begin.hour < 7 {
      begin.hour += 12;
    }
    if end.hour < 7 {
      end.hour += 12;
    }
  }

  let begin_minutes = begin.hour * 60 + begin.minute;
  let end_minutes = end.hour * 60 + end.minute;

  return Ok((begin_minutes, end_minutes));
}
