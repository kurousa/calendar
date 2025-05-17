use chrono::NaiveDateTime;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Schedule {
    id: u64,
    subject: String,
    start: NaiveDateTime,
    end: NaiveDateTime,
}
impl Schedule {
    /// 予定の重複を確認
    fn intersects(&self, other: &Schedule) -> bool {
        self.start < other.end && other.start < self.end
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Calendar {
    schedules: Vec<Schedule>,
}

#[derive(Subcommand)]
enum Commands {
    List,
    Add {
        subject: String,
        start: NaiveDateTime,
        end: NaiveDateTime,
    },
    Delete {
        id: u64,
    },
}

enum MyError {
    Io(std::io::Error),
    Serde(serde_json::Error),
}
impl From<std::io::Error> for MyError {
    fn from(err: std::io::Error) -> MyError {
        MyError::Io(err)
    }
}
impl From<serde_json::Error> for MyError {
    fn from(err: serde_json::Error) -> MyError {
        MyError::Serde(err)
    }
}
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
const SCHEDULE_FILE: &str = "schedules.json";
/// スケジュールファイル新規作成
fn create_schedule_file() -> Result<(), MyError> {
    let file = File::create(SCHEDULE_FILE)?;
    let writer = BufWriter::new(file);
    let empty_calendar = Calendar { schedules: vec![] };
    serde_json::to_writer(writer, &empty_calendar)?;
    Ok(())
}

/// スケジュールファイルを読み込む
fn read_calendar() -> Result<Calendar, MyError> {
    let file = File::open(SCHEDULE_FILE)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader).unwrap())
}
/// スケジュールファイルに書き込む
fn save_calendar(calendar: &Calendar) -> Result<(), MyError> {
    let file = File::create(SCHEDULE_FILE)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, calendar)?;
    Ok(())
}
/// スケジュールの一覧を表示
fn show_schedule(calendar: &Calendar) {
    println!("ID\tSTART\tEND\tSUBJECT");
    for schedule in &calendar.schedules {
        println!(
            "{}\t{}\t{}\t{}",
            schedule.id,
            schedule.start.format("%Y-%m-%d %H:%M"),
            schedule.end.format("%Y-%m-%d %H:%M"),
            schedule.subject
        );
    }
}
/// スケジュールを追加
fn add_schedule(
    calendar: &mut Calendar,
    subject: String,
    start: NaiveDateTime,
    end: NaiveDateTime,
) -> bool {
    let id = calendar.schedules.len() as u64;
    let new_schedule = Schedule {
        id,
        subject,
        start,
        end,
    };

    for schedule in &calendar.schedules {
        println!("schedule.start:{:?}", schedule.start);
        println!("new_schedule.end:{:?}", new_schedule.end);
        if schedule.intersects(&new_schedule) {
            println!(
                "新しい予定:{:?}は,既存の予定:{:?}と重複しています。",
                new_schedule, schedule
            );
            return false;
        }
    }
    calendar.schedules.push(new_schedule);
    true
}
/// スケジュールを削除
fn delete_schedule(calendar: &mut Calendar, id: u64) -> bool {
    for i in 0..calendar.schedules.len() {
        if calendar.schedules[i].id == id {
            calendar.schedules.remove(i);
            return true;
        }
    }
    false
}

/// メイン関数
fn main() {
    let options = Cli::parse();

    match options.command {
        Commands::List => match read_calendar() {
            Ok(calendar) => show_schedule(&calendar),
            Err(_) => {
                println!("カレンダーの読み込みエラーが発生しました、ファイル{SCHEDULE_FILE}が存在しないか、権限がありません。");
            }
        },
        Commands::Add {
            subject,
            start,
            end,
        } => match read_calendar() {
            Ok(mut calendar) => {
                if add_schedule(&mut calendar, subject, start, end) {
                    match save_calendar(&calendar) {
                        Ok(_) => println!("予定を保存しました"),
                        Err(_) => println!("予定の保存に失敗しました"),
                    };
                } else {
                    println!("予定を追加できませんでした");
                }
            }
            Err(_) => match create_schedule_file() {
                Ok(_) => {
                    println!("既存のカレンダーが存在しないため、新しいカレンダーを作成しました。");
                    let mut calendar = Calendar { schedules: vec![] };
                    if add_schedule(&mut calendar, subject, start, end) {
                        match save_calendar(&calendar) {
                            Ok(_) => println!("予定を保存しました"),
                            Err(_) => println!("予定の保存に失敗しました"),
                        };
                    } else {
                        println!("予定を追加できませんでした");
                    }
                }
                Err(_) => {
                    println!("カレンダーの作成エラーが発生しました");
                }
            },
        },
        Commands::Delete { id } => match read_calendar() {
            Ok(mut calendar) => {
                if delete_schedule(&mut calendar, id) {
                    match save_calendar(&calendar) {
                        Ok(_) => println!("予定を保存しました"),
                        Err(_) => println!("予定の保存に失敗しました"),
                    };
                    println!("予定を削除しました");
                } else {
                    println!("予定を削除できませんでした");
                }
            }
            Err(_) => {
                println!("カレンダーの読み込みエラーが発生しました、ファイル{SCHEDULE_FILE}が存在しないか、権限がありません。");
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn naive_date_time(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        seconds: u32,
    ) -> NaiveDateTime {
        chrono::NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(hour, minute, seconds)
            .unwrap()
    }

    /// 予定重複テストヘルパー
    fn test_schedule_intersects_helper(
        h0: u32,
        mo: u32,
        h1: u32,
        m1: u32,
        expect_intersects: bool,
    ) {
        let existing_schedule = Schedule {
            id: 1,
            subject: "既存の予定".to_string(),
            start: naive_date_time(2024, 1, 1, h0, mo, 0),
            end: naive_date_time(2024, 1, 1, h1, m1, 0),
        };

        let new_schedule = Schedule {
            id: 999,
            subject: "新しい予定".to_string(),
            start: naive_date_time(2024, 1, 1, 19, 0, 0),
            end: naive_date_time(2024, 1, 1, 20, 0, 0),
        };

        assert_eq!(
            expect_intersects,
            existing_schedule.intersects(&new_schedule),
        );
    }
    #[test]
    /// 既存の予定実施中に、新しい予定が開始するパターン
    /// - 既存の予定：2024年1月1日18時15分から19時15分
    /// - 新しい予定：2024年1月1日18時30分から20時0分
    /// - 想定結果：重複する
    fn test_schedule_intersects_1() {
        let existing_schedule = Schedule {
            id: 1,
            subject: "既存の予定".to_string(),
            start: naive_date_time(2024, 1, 1, 18, 15, 0),
            end: naive_date_time(2024, 1, 1, 19, 15, 0),
        };

        let new_schedule = Schedule {
            id: 2,
            subject: "新しい予定".to_string(),
            start: naive_date_time(2024, 1, 1, 18, 30, 0),
            end: naive_date_time(2024, 1, 1, 20, 0, 0),
        };

        assert!(existing_schedule.intersects(&new_schedule));
    }

    #[test]
    /// 新しい予定実施中に、既存の予定が開始するパターン
    /// - 既存の予定：2024年1月1日19時45分から20時45分
    /// - 新しい予定：2024年1月1日19時00分から20時00分
    /// - 想定結果：重複する
    fn test_schedule_intersects_2() {
        let existing_schedule = Schedule {
            id: 1,
            subject: "既存の予定".to_string(),
            start: naive_date_time(2024, 1, 1, 19, 45, 0),
            end: naive_date_time(2024, 1, 1, 20, 45, 0),
        };

        let new_schedule = Schedule {
            id: 2,
            subject: "新しい予定".to_string(),
            start: naive_date_time(2024, 1, 1, 19, 0, 0),
            end: naive_date_time(2024, 1, 1, 20, 0, 0),
        };

        assert!(existing_schedule.intersects(&new_schedule));
    }

    #[test]
    /// 既存の予定実施中に、新しい予定が開始し終了するパターン
    /// - 既存の予定：2024年1月1日18時30分から20時15分
    /// - 新しい予定：2024年1月1日19時00分から20時00分
    /// - 想定結果：重複する
    fn test_schedule_intersects_3() {
        let existing_schedule = Schedule {
            id: 1,
            subject: "既存の予定".to_string(),
            start: naive_date_time(2024, 1, 1, 18, 30, 0),
            end: naive_date_time(2024, 1, 1, 20, 15, 0),
        };

        let new_schedule = Schedule {
            id: 2,
            subject: "新しい予定".to_string(),
            start: naive_date_time(2024, 1, 1, 19, 0, 0),
            end: naive_date_time(2024, 1, 1, 20, 0, 0),
        };

        assert!(existing_schedule.intersects(&new_schedule));
    }

    #[test]
    /// 既存の予定と新しい予定は重複しないパターン(同日)
    /// - 既存の予定：2024年1月1日20時15分から20時45分
    /// - 新しい予定：2024年1月1日19時00分から20時00分
    /// - 想定結果：重複しない
    fn test_schedule_intersects_4() {
        let existing_schedule = Schedule {
            id: 1,
            subject: "既存の予定".to_string(),
            start: naive_date_time(2024, 1, 1, 20, 15, 0),
            end: naive_date_time(2024, 1, 1, 20, 45, 0),
        };

        let new_schedule = Schedule {
            id: 2,
            subject: "新しい予定".to_string(),
            start: naive_date_time(2024, 1, 1, 19, 0, 0),
            end: naive_date_time(2024, 1, 1, 20, 0, 0),
        };

        assert!(!existing_schedule.intersects(&new_schedule));
    }

    #[test]
    /// 既存の予定と新しい予定は重複しないパターン2(別日)
    /// - 既存の予定：2023年12月8日09時00分から10時30分
    /// - 新しい予定：2024年12月15日10時00分から11時00分
    /// - 想定結果：重複しない
    fn test_schedule_intersects_5() {
        let existing_schedule = Schedule {
            id: 1,
            subject: "既存の予定".to_string(),
            start: naive_date_time(2023, 12, 8, 9, 0, 0),
            end: naive_date_time(2023, 12, 8, 10, 30, 0),
        };

        let new_schedule = Schedule {
            id: 2,
            subject: "新しい予定".to_string(),
            start: naive_date_time(2023, 12, 15, 10, 0, 0),
            end: naive_date_time(2023, 12, 15, 11, 0, 0),
        };

        assert!(!existing_schedule.intersects(&new_schedule));
    }

    #[test]
    fn test_schedule_intersects_with_helper() {
        // 既存の予定：2024年1月1日18時15分から19時15分
        // 新しい予定：2024年1月1日18時30分から20時0分
        // 想定結果：重複する
        test_schedule_intersects_helper(18, 15, 19, 15, true);
        // 既存の予定：2024年1月1日19時45分から20時45分
        // 新しい予定：2024年1月1日19時00分から20時00分
        // 想定結果：重複する
        test_schedule_intersects_helper(19, 45, 20, 45, true);
        // 既存の予定：2024年1月1日18時30分から20時15分
        // 新しい予定：2024年1月1日19時00分から20時00分
        // 想定結果：重複する
        test_schedule_intersects_helper(18, 30, 20, 15, true);
    }

    use rstest::rstest;
    #[rstest]
    #[case::no_intersects_before(18, 15, 18, 45, false)]
    #[case::intersects_start(18, 15, 19, 15, true)]
    #[case::intersects_end(19, 15, 19, 45, true)]
    #[case::no_intersects_after(20, 15, 20, 45, false)]
    fn test_schedule_intersects_with_rstest(
        #[case] h0: u32,
        #[case] mo: u32,
        #[case] h1: u32,
        #[case] m1: u32,
        #[case] expect_intersects: bool,
    ) {
        let existing_schedule = Schedule {
            id: 1,
            subject: "既存の予定".to_string(),
            start: naive_date_time(2024, 1, 1, h0, mo, 0),
            end: naive_date_time(2024, 1, 1, h1, m1, 0),
        };
        let new_schedule = Schedule {
            id: 999,
            subject: "新しい予定".to_string(),
            start: naive_date_time(2024, 1, 1, 19, 0, 0),
            end: naive_date_time(2024, 1, 1, 20, 0, 0),
        };
        assert_eq!(
            expect_intersects,
            existing_schedule.intersects(&new_schedule),
        );
    }

    #[test]
    fn test_delete_schedule() {
        let mut calendar = Calendar {
            schedules: vec![
                Schedule {
                    id: 0,
                    subject: "予定1".to_string(),
                    start: naive_date_time(2024, 1, 1, 18, 0, 0),
                    end: naive_date_time(2024, 1, 1, 19, 0, 0),
                },
                Schedule {
                    id: 1,
                    subject: "予定2".to_string(),
                    start: naive_date_time(2024, 1, 1, 20, 0, 0),
                    end: naive_date_time(2024, 1, 1, 21, 0, 0),
                },
                Schedule {
                    id: 2,
                    subject: "予定3".to_string(),
                    start: naive_date_time(2024, 1, 1, 22, 0, 0),
                    end: naive_date_time(2024, 1, 1, 23, 0, 0),
                },
            ],
        };

        assert!(delete_schedule(&mut calendar, 0));
        // 想定するid:0の予定を削除後のカレンダー
        let expected = Calendar {
            schedules: vec![
                Schedule {
                    id: 1,
                    subject: "予定2".to_string(),
                    start: naive_date_time(2024, 1, 1, 20, 0, 0),
                    end: naive_date_time(2024, 1, 1, 21, 0, 0),
                },
                Schedule {
                    id: 2,
                    subject: "予定3".to_string(),
                    start: naive_date_time(2024, 1, 1, 22, 0, 0),
                    end: naive_date_time(2024, 1, 1, 23, 0, 0),
                },
            ],
        };
        assert_eq!(calendar.schedules.len(), 2);
        assert_eq!(calendar, expected);

        assert!(delete_schedule(&mut calendar, 1));
        // 想定するid:1の予定を削除後のカレンダー
        let expected = Calendar {
            schedules: vec![Schedule {
                id: 2,
                subject: "予定3".to_string(),
                start: naive_date_time(2024, 1, 1, 22, 0, 0),
                end: naive_date_time(2024, 1, 1, 23, 0, 0),
            }],
        };
        assert_eq!(calendar.schedules.len(), 1);
        assert_eq!(calendar, expected);

        assert!(delete_schedule(&mut calendar, 2));
        // 想定するid:1の予定を削除後のカレンダー
        let expected = Calendar { schedules: vec![] };
        assert_eq!(calendar.schedules.len(), 0);
        assert_eq!(calendar, expected);
    }
}
