use std::collections::HashMap;
use std::fs;
use std::time::{Instant, Duration};
use ordered_float::OrderedFloat;

#[derive(Debug, Clone)]
pub struct Arrow {
    pub lane: usize,
    pub hit_time: f64,  
    pub spawn_time: f64,
    pub long: bool,
    pub end_time: f64,
}

pub struct GameState {
    pub difficulties: Vec<String>,
    pub start_time: Instant,
    pub arrows: Vec<Arrow>,
    
    pub song: String,
    pub artist: String,
    pub title: String,
    pub bg: String,

    pub sample_start: f64,
    pub sample_len: f64,

    all_arrows: Vec<Vec<Arrow>>
}



impl GameState {
    pub fn new(file_path: &str, speed: f64) -> Self {
        let contents = fs::read_to_string(file_path).expect("Failed to read file");
        let (mut all_arrows,
             difficulties,
             song, 
             artist, 
             title, 
             bg, 
             sample_start,
             sample_len) = Self::parse_sm_file(&contents);

        for arrows in &mut all_arrows {
            for arrow in arrows {
                arrow.spawn_time = arrow.hit_time - speed;
            }
        }

        let arrows: Vec<Arrow> = Vec::new();

        Self {
            difficulties,
            start_time: Instant::now(),
            arrows,
            song,
            artist,
            title,
            bg,
            sample_start,
            sample_len,
            all_arrows
        }
    }

    pub fn start(&mut self, difficulty: u32){
        let arrows: Vec<Arrow> = self.all_arrows
            .get(difficulty as usize)   
            .cloned()
            .unwrap_or_default();

        let mut start_time = 1.0;
        if let Some(arrow) = arrows.first(){
            if arrow.spawn_time < 0.0 {
                start_time -= arrow.spawn_time;
            }

        }


        self.start_time = Instant::now() + Duration::from_secs_f64(start_time);
        self.arrows = arrows;

    }


    fn parse_sm_file(content: &str) -> (Vec<Vec<Arrow>>, Vec<String>, String, String, String, String, f64, f64) {
        let mut bpm_map: HashMap<OrderedFloat<f64>, f64> = HashMap::new();
        let mut stop_map: HashMap<OrderedFloat<f64>, f64> = HashMap::new();
        let mut measures: Vec<Vec<Vec<&str>>> = Vec::new();
        let mut current_measure: Vec<&str> = Vec::new();
        let mut all_arrows: Vec<Vec<Arrow>> = Vec::new();
        let mut difficulties: Vec<String> = Vec::new();

        let title = Self::parse_tag_value(content, "#TITLE:").unwrap_or_default();
        let artist = Self::parse_tag_value(content, "#ARTIST:").unwrap_or_default();
        let song = Self::parse_tag_value(content, "#MUSIC:").unwrap_or_default();
        let bg = Self::parse_tag_value(content, "#BACKGROUND:").unwrap_or_default();
        let offset: f64 = Self::parse_tag_value(content, "#OFFSET:").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0).abs();
        let sample_start = Self::parse_tag_value(content, "#SAMPLESTART:").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
        let sample_len = Self::parse_tag_value(content, "#SAMPLELENGTH:").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);




        if let Some(bpm_index) = content.find("#BPMS"){
            for entry in content[bpm_index+6..].split(";").next().unwrap_or("").replace("\n","").split(",") {
                let parts: Vec<&str> = entry.split('=').collect();
                if parts.len() <= 1 {
                    continue;
                }

                if let (Ok(beat), Ok(bpm)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                    bpm_map.insert(OrderedFloat(beat), bpm);
                }
            }            
        }
        if let Some(bpm_index) = content.find("#STOPS"){
            for entry in content[bpm_index+7..].split(";").next().unwrap_or("").replace("\n","").split(",") {
                let parts: Vec<&str> = entry.split('=').collect();
                if parts.len() <= 1 {
                    continue;
                }

                if let (Ok(beat), Ok(bpm)) = (parts[0].parse::<f64>(), parts[1].trim().parse::<f64>()) {
                    stop_map.insert(OrderedFloat(beat), bpm);

                }
                

            }            
        }

        let mut parsing = false;
        let mut lines_since_notes = 0;
        let mut current_measure_difficulty: Vec<Vec<&str>> = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("#NOTES") {
                lines_since_notes = 0;
                parsing = true;

                if !current_measure_difficulty.is_empty(){
                    measures.push(current_measure_difficulty);
                    current_measure_difficulty = Vec::new();
                }

            } else {
                if !parsing {
                    continue;
                }

                lines_since_notes += 1;
                if lines_since_notes == 3 {
                    difficulties.push(line.trim_end_matches(':').to_string());
                }

                if line.starts_with(",") {
                    if !current_measure.is_empty() {
                        current_measure_difficulty.push(current_measure);
                        current_measure = Vec::new();
                    }
                } else if !line.is_empty() {
                    if line.chars().all(|c| c == '0' || c == '1' || c == '2' || c=='3') {
                        current_measure.push(line);
                    }
                }
            }
        }

        
        if !current_measure.is_empty() {
            current_measure_difficulty.push(current_measure);
        }
        if !current_measure_difficulty.is_empty(){
            measures.push(current_measure_difficulty);
        }


        for difficulty in measures{
            let mut measure_index = 0;
            let mut last_long_start = [0.0,0.0,0.0,0.0];
            let mut cur_arrows: Vec<Arrow> = Vec::new();

            for measure in difficulty {
                let num_lines = measure.len();
                if num_lines == 0 {
                    continue;
                }
                let beat_increment = 4.0 / (num_lines as f64);
                for (line_index, line) in measure.iter().enumerate() {

                    let current_beat = (measure_index as f64) * 4.0 + (line_index as f64) * beat_increment;

                    for (lane, ch) in line.chars().enumerate() {
                        if ch == '1' {
                            let hit_time = Self::beat_to_time(current_beat, &bpm_map, offset, &stop_map);
                            cur_arrows.push(Arrow {
                                lane,
                                hit_time,
                                spawn_time: 0.0,
                                long: false,
                                end_time: 0.0,
                            });
                        } else if ch == '2' {
                            last_long_start[lane] = Self::beat_to_time(current_beat, &bpm_map, offset, &stop_map);
                        } else if ch == '3' {
                            let end_time = Self::beat_to_time(current_beat, &bpm_map, offset, &stop_map);
                            cur_arrows.push(Arrow {
                                lane,
                                hit_time: last_long_start[lane],
                                spawn_time: 0.0,
                                long: true,
                                end_time,
                            });                        
                        }
                    }
                }
                measure_index += 1;
            }
            all_arrows.push(cur_arrows);
        }


        (all_arrows, difficulties, song, artist, title, bg, sample_start, sample_len)
    }

    fn beat_to_time(beat: f64, bpm_map: &HashMap<OrderedFloat<f64>, f64>, offset: f64, stop_map: &HashMap<OrderedFloat<f64>, f64>) -> f64 {
        let mut last_time = offset;
        let mut last_beat = 0.0;
        let mut last_bpm = 120.0;

        let mut beats: Vec<f64> = bpm_map.keys().map(|k| k.0).collect();
        beats.sort_by(|a, b| a.partial_cmp(b).unwrap());

        for &bpm_beat in &beats {
            let bpm = bpm_map.get(&OrderedFloat(bpm_beat)).unwrap();
            if beat < bpm_beat {
                break;
            }
            last_time += (bpm_beat - last_beat) * (60.0 / last_bpm);
            last_beat = bpm_beat;
            last_bpm = *bpm;
        }



        for (&stop_beat, &stop_duration) in stop_map.iter(){
            if f64::from(stop_beat) <= beat{
                last_time += stop_duration;
            }
        }

        last_time + (beat - last_beat) * (60.0 / last_bpm)
    }


    fn parse_tag_value(contents: &str, tag: &str) -> Option<String> {
        if let Some(start) = contents.find(tag) {
            let rest = &contents[start + tag.len()..];
            if let Some(end) = rest.find(';') {
                return Some(rest[..end].trim().to_string());
            }
        }
        None
    }



}
