use modql::filter::ListOptions;
use serde::{de::DeserializeOwned, Deserialize};
use serde_with::{serde_as, OneOrMany};

/// All apis that want to create something
#[derive(Deserialize)]
pub struct ParamsForCreate<D> {
    pub data: D,
}

/// All apis that want to update something
#[derive(Deserialize)]
pub struct ParamsForUpdate<D> {
    pub id: i64,
    pub data: D,
}

/// All apis that only need the id
#[derive(Deserialize)]
pub struct ParamsIded {
    pub id: i64,
}

#[serde_as]
#[derive(Deserialize)]
pub struct ParamsList<F>
where
    // F owns the data that is deserialized
    F: DeserializeOwned,
{
    // https://docs.rs/serde_with/latest/serde_with/struct.OneOrMany.html
    // will allow to pass unique element without [] in json
    #[serde_as(deserialize_as = "Option<OneOrMany<_>>")]
    pub filters: Option<Vec<F>>,
    pub list_options: Option<ListOptions>,
}
