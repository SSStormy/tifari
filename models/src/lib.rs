use std::collections::HashSet;

#[macro_use]
extern crate serde_derive;
extern crate serde;

#[derive(Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Tag
{
    id: i64,
    name: String,
}

impl Tag {
    pub fn new(id: i64, name: String) -> Self {
        Tag { id, name }
    }

    pub fn get_id(&self) -> i64 { self.id }
    pub fn get_name(&self) -> &String { &self.name }
}

#[derive(Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TagWithUsage {
    id: i64,
    name: String,
    times_used: i64,
}

impl TagWithUsage {
    pub fn new(id: i64, name: String, times_used: i64) -> Self {
        TagWithUsage { id, name, times_used }
    }

    pub fn get_id(&self) -> i64 { self.id }
    pub fn get_name(&self) -> &String { &self.name }
    pub fn get_times_used(&self) -> i64 { self.times_used }
}

#[derive(Serialize, Deserialize)]
pub struct Image
{
    id: i64,
    path: String,
    created_at_time : i64,
    tags: HashSet<Tag>,
}

impl Image {
    pub fn new(id: i64, path: String, created_at_time: i64, tags: HashSet<Tag>) -> Self {
        Image { id, path, created_at_time, tags }
    }
    pub fn new_no_tags(id: i64, path: String, created_at_time: i64) -> Self {
        Image { id, path, created_at_time, tags: HashSet::new() }
    }

    pub fn get_id(&self) -> i64 { self.id }
    pub fn get_path(&self) -> &String { &self.path }
    pub fn get_created_at_time(&self) -> i64 { self.created_at_time }
    pub fn get_tags(&self) -> &HashSet<Tag> { &self.tags }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    status: u32,
    message: String
}

impl ErrorResponse {
    pub fn new(status: u32, message: String) -> Self {
        ErrorResponse { status, message }
    }

    pub fn get_status(&self) -> u32 { self.status }
    pub fn get_message(&self) -> &String { &self.message }
}

#[derive(Deserialize)]
pub struct AddTagsRequest {
    tags: Vec<String>,
    image_ids: Vec<i64>
}

impl AddTagsRequest {
    pub fn get_tags(&self) -> &Vec<String> { &self.tags }
    pub fn get_image_ids(&self) -> &Vec<i64> { &self.image_ids }
}

#[derive(Deserialize)] 
pub struct RemoveTagsRequest {
    tag_ids: Vec<i64>,
    image_ids: Vec<i64>,
}

impl RemoveTagsRequest {
    pub fn get_tag_ids(&self) -> &Vec<i64> { &self.tag_ids } 
    pub fn get_image_ids(&self) -> &Vec<i64> { &self.image_ids } 
}

#[derive(Serialize)]
pub struct TagQueueSizeResponse {
    tag_queue_size: i64,
}

impl TagQueueSizeResponse {
    pub fn new(tag_queue_size: i64) -> Self {
        TagQueueSizeResponse { tag_queue_size }
    }
}
