#![feature(fn_traits)]
extern crate core;

use std::num::ParseIntError;
use std::str::FromStr;
use surf::{Client, Config};
use surf::http::headers::HeaderValue;

pub enum DeserializationError {
    ParseIntErr(ParseIntError),
    UnexpectedChar { expected: String, found: char }
}

trait Day {
    fn solve(&self, input: String, part: DayPart) -> String;
}

pub trait DayImpl {
    type Input<'a>;
    type Output;
    fn deserialize_input<'a>(&self, input: &'a str) -> Self::Input<'a>;
    fn serialize_output(&self, output: Self::Output) -> String;
    fn solve_first(&self, input: Self::Input<'_>) -> Self::Output;
    fn solve_second(&self, input: Self::Input<'_>) -> Self::Output;
}

impl<T> Day for T where T: DayImpl {
    fn solve(&self, input: String, part: DayPart) -> String {
        let solver = match part {
            DayPart::FIRST => DayImpl::solve_first,
            DayPart::SECOND => DayImpl::solve_second,
        };
        let input = self.deserialize_input(&input);
        let output = solver.call((self, input));
        self.serialize_output(output)
    }
}

struct UnimplementedDay;

impl Day for UnimplementedDay {
    fn solve(&self, input: String, part: DayPart) -> String {
        unimplemented!()
    }
}

pub struct AocAccount {
    client: Client
}

impl AocAccount {
    pub fn new(token: &str) -> surf::Result<Self> {
        Ok(AocAccount {
            client: Config::new()
                .add_header("Cookie", HeaderValue::from_str(&format!("session={token}"))?)?
                .try_into()?
        })
    }
}

pub struct AocYear {
    account: AocAccount,
    year: u16,
    days: [&'static dyn Day; 25]
}

impl AocYear {
    pub fn new(account: AocAccount, year: u16) -> Self {
        AocYear { account, year, days: [&UnimplementedDay; 25] }
    }
    pub fn add(&mut self, index: usize, day: &'static dyn Day) {
        self.days[index] = day
    }
}

pub enum DayPart { FIRST, SECOND }

impl FromStr for DayPart {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(DayPart::FIRST),
            "2" => Ok(DayPart::SECOND),
            _ => Err("NoSuchPart".to_string())
        }
    }
}

pub enum Error {
    InvalidDay,
    FailedToRequestInput(surf::Error),
    DeserializationError(DeserializationError),
}

impl AocYear {
    async fn get_input_string(&self, day: u8) -> surf::Result<String> {
        Ok(self.account.client
            .get(format!("https://adventofcode.com/{}/day/{}/input", self.year, day))
            .recv_string().await?
            .trim_end_matches("\n").to_string())
    }
    pub async fn solve(&self, day: u8, part: DayPart) -> Result<String, Error> {
        if day == 0 || day > 25 { return Err(Error::InvalidDay) }
        let input = self.get_input_string(day).await
            .map_err(|err| Error::FailedToRequestInput(err))?;
        self.days[day as usize - 1].solve(input, part)
    }
}
