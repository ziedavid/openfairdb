// Copyright (c) 2015 Markus Kohlhase <mail@markus-kohlhase.de>

use geo;

use json::{Entry, Category};
use geo::Coordinate;
use std::cmp::min;
use std::collections::HashSet;
use std::f64::consts;


pub trait Search {
    fn filter_by_search_text(&self, text: &str) -> Self;
    fn map_to_ids(&self) -> Vec<String>;
}

impl Search for Vec<Entry> {
    fn filter_by_search_text(&self, text: &str) -> Vec<Entry> {
        by_text(self, text, entry_filter_factory)
    }
    fn map_to_ids(&self) -> Vec<String> {
        self.iter().cloned().filter_map(|e| e.clone().id).collect()
    }
}

impl Search for Vec<Category> {
    fn filter_by_search_text(&self, text: &str) -> Vec<Category> {
        by_text(self, text, category_filter_factory)
    }
    fn map_to_ids(&self) -> Vec<String> {
        self.iter().cloned().filter_map(|e| e.clone().id).collect()
    }
}

fn by_text<'a, T: Clone, F>(collection: &'a [T], text: &str, filter: F) -> Vec<T>
    where F: Fn(&'a T) -> Box<Fn(&str) -> bool + 'a>
{
    let txt_cpy = text.to_lowercase();
    let words = txt_cpy.split(',');
    collection.iter()
        .filter(|&e| {
            let f = filter(e);
            words.clone().any(|word| f(word))
        })
        .cloned()
        .collect()
}

fn entry_filter_factory<'a>(e: &'a Entry) -> Box<Fn(&str) -> bool + 'a> {
    Box::new(move |word| {
        e.title.to_lowercase().contains(word) || e.description.to_lowercase().contains(word)
    })
}

fn category_filter_factory<'a>(e: &'a Category) -> Box<Fn(&str) -> bool + 'a> {
    Box::new(move |word| match e.name {
        Some(ref n) => n.to_lowercase().contains(word),
        None => false,
    })
}






// FIND DUPLICATES:


// return vector of entries like: (entry1, entry2, reason) where entry1 and entry2 are similar entries
pub fn find_duplicates(entries: &Vec<Entry>) -> Vec<(&Entry, &Entry, &str)> {
  let mut duplicates = Vec::new();
  for i in 0..entries.len() {
    for j in (i+1)..entries.len() {
      match is_duplicate(&entries[i], &entries[j]){
        Some(DuplicateType::SimilarChars) => duplicates.push((&entries[i], &entries[j], "similar title (characters)")),
        Some(DuplicateType::SimilarWords) => duplicates.push((&entries[i], &entries[j], "similar title (words)")),
        None => {},
      };

    }
  }

  for dup in &duplicates {
    info!("{} - {} ({})", dup.0.title, dup.1.title, dup.2);
  }

  duplicates
}

#[derive(Debug)]
enum DuplicateType {
    SimilarChars,
    SimilarWords
}
// returns 0 if e1 and e2 are not similar_titl
// returns 1 if the hamming distance of their titles is small
// returns 2 if at most one word of their titles is different (and they don't both have one-word titles)
// returns 3 if they are located within 10 meters of each other
fn is_duplicate(e1 : &Entry, e2: &Entry) -> Option<DuplicateType>{
  //println!("\ncomparing '{}' and '{}':", e1.title, e2.title);

  if similar_title(&e1, &e2, 0.1, 0) && in_close_proximity(&e1, &e2, 100) {Some(DuplicateType::SimilarChars)} 
  else if similar_title(&e1, &e2, 0.0, 1) && in_close_proximity(&e1, &e2, 100) {Some(DuplicateType::SimilarWords)}
  else {None}  // entries are not similar 
}



fn in_close_proximity(e1: &Entry, e2: &Entry, maxDistMeters:u32) -> bool{
    let dist = entry_distance_in_meters(&e1, &e2) as f32;
    if dist <= maxDistMeters as f32{
      println!("lat1: {}, lat2: {}, lng1: {}, lng2: {} --> distance: {} m (near)", e1.lat, e2.lat, e1.lng, e2.lng, dist);
      true
    } else {
      //println!("distance: {} --> far", dist);
      false
    }
}


fn entry_distance_in_meters(e1: &Entry, e2: &Entry) -> f64{
  let coord1 = Coordinate{lat: e1.lat, lng:e1.lng};
  let coord2 = Coordinate{lat: e2.lat, lng:e2.lng};
  geo::distance(&coord1, &coord2) * 1000.0
}


fn similar_title(e1: &Entry, e2: &Entry, maxPercentDifferent: f32, maxWordsDifferent: u32) -> bool{
  let maxDist : usize = ((min(e1.title.len(),e2.title.len()) as f32 * maxPercentDifferent) + 1.0) as usize;  // +1 is to get the ceil
  if hamming_distance_small(&e1.title, &e2.title, maxDist) | (words_equal_except_k_words(&e1.title, &e2.title, maxWordsDifferent)) {
    //println!("--> titles similar");
    true 
  } else {
    //println!("--> titles not similar");
    false
  }
}




// returns true if all but k words are equal in str1 and str2 
// (words in str1 and str2 are treated as sets, order & multiplicity of words doesn't matter)
fn words_equal_except_k_words(str1: &str, str2:&str, k:u32) -> bool{
  let mut s1 : &str;
  let mut s2 : &str;

  let len1 =  str1.split_whitespace().count();
  let len2 =  str2.split_whitespace().count();

  if (len1 == 1) & (len2 == 1) {
    false
  } else{

      if len1 <= len2 {
        s1 = str1;
        s2 = str2;
      } else{
        s1 = str2;
        s2 = str1;
      }

      let mut words1 = s1.split_whitespace();
      let mut words2 = s2.split_whitespace();

      let mut diff = 0;
      let mut set1 = HashSet::new();
      for w in words1 {
        set1.insert(w);
      }

      let mut words2 = s2.split(" ");
      for w in words2 {
        if !set1.contains(w) {
          diff = diff + 1;
        }
      }
      
      
      // return:
      //println!("words different: {}/{} --> {}", diff, k, if diff <= k {"similar"} else {"not similar"});
      diff <= k
  }
}




// returns true if the hamming distance between str1 and str2 is smaller or equal as maxDist
// (doesn't need to calculate the full hamming distance because it aborts as soon as maxDist is reached)
fn hamming_distance_small(str1: &str, str2:&str, maxDist:usize) -> bool{
  let mut dist = 0;
  //println!("comparing:\n{}\n{}", str1, str2);
  for i in 0..str1.chars().count() {
    if str1.chars().nth(i) != str2.chars().nth(i) {
      dist = dist + 1;
      if dist > maxDist {
        break;
      }
    }
  }

  if dist > maxDist{
    //println!("hamming distance larger than {} --> not similar", maxDist);
    false
  } else{
    //println!("hamming distance {}/{} --> similar", dist, maxDist);
    true
  }
}


// Levenshtein Distance more realistically captures typos (all of the following operations are counted as distance 1: add one character in between, delete one character, change one character)
// but it proved to be way too slow to be run on the whole dataset
fn levenshtein_distance(s: &str, t:&str) -> usize{
  levenshtein_distance2(s, s.len(), t, t.len())
}

// https://en.wikipedia.org/wiki/Levenshtein_distance#Computing_Levenshtein_distance
fn levenshtein_distance2(s: &str, len_s : usize, t: &str, len_t: usize) -> usize {
  let cost : usize;
  //println!("{} - {}",s, len_s);
  //println!("{} - {}",t, len_t);

  // base case: empty strings
  if len_s == 0 {
      //println!("s is empty");
      len_t
  } else {
    if len_t == 0 {
        //println!("t is empty");
        len_s
      } else {

      // test if last characters of the strings match 
      if s.chars().nth(len_s-1) == t.chars().nth(len_t-1) {cost = 0;} else {cost = 1;}
      // return minimum of delete char from s, delete char from t, and delete char from both
      min3(levenshtein_distance2(s, len_s - 1, t, len_t) + 1, levenshtein_distance2(s, len_s, t, len_t - 1) + 1, levenshtein_distance2(s, len_s - 1, t, len_t - 1) + cost)
     }
  }
}


fn min3(s:usize, t:usize, u:usize) -> usize{
  if s <= t {
    min(s,u)
  } else{
    min(t,u)
  }
}