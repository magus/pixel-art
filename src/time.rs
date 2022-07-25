use std::time::Instant;
#[derive(Debug)]
pub struct Record<'a> {
    label: &'a str,
    time: Instant,
}

#[derive(Debug)]
pub struct Stopwatch<'a> {
    records: Vec<Record<'a>>,
}

impl<'a> Stopwatch<'a> {
    pub fn start() -> Stopwatch<'a> {
        let mut records = vec![];

        records.push(Record {
            label: "start",
            time: Instant::now(),
        });

        return Stopwatch { records };
    }

    pub fn record(&mut self, label: &'a str) {
        let last_record = self.records.last().unwrap();
        let duration = last_record.time.elapsed();

        self.records.push(Record {
            label,
            time: Instant::now(),
        });

        println!("\n⏱️ [time::{}] {:?}", label, duration);
    }

    pub fn all(&self) {
        let mut start = self.records.first().unwrap().time;

        println!("\n⏱️ [times::all]");

        for i in 1..self.records.len() {
            let record = self.records.get(i).unwrap();
            let duration = record.time.duration_since(start);
            println!("   [{}] {:?}", record.label, duration);

            start = record.time;
        }
    }
}
