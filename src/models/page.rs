#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemsPerPage(i32);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageNumber(i32);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TotalPages(i32);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub items_per_page: ItemsPerPage,
    pub page_number: PageNumber,
    pub total_pages: TotalPages,
}
