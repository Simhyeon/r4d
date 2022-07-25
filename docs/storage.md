### Storage

Storage features provdies a single trait of ```RadStorage``` which implements
```update``` and ```extract```.

You can call update and extract method with update and extract macro
respectively.

#### Example

You need to import [Serde](https://github.com/serde-rs/serde) in your proejct
for stroage.

Create storage struct

```rust
use serde::{Serialize,Deserialize};
use r4d::{StorageOutput, StorageResult};

#[derive(Serialize,Deserialize)]
pub struct PlotModel {
	// informations
	...
}

impl RadStorage for PlotModel {
	fn update(&mut self, args : &[String]) -> StorageResult<()> {
		// Update logics
	}

	// $extract() macro calls extract method with the serialize value of "false"
	fn extract(&mut self, serialize: bool) -> StorageResult<Option<StorageOutput>> {
		if serialize {
			Ok(Some(StorageOutput::Binary(bincode::serialize(self)?)))
		} else {
			Ok(Some(StorageOutput::Text(serde_json::to_string(self)?)))
		}
	}
}
```
And add storage to processor before processing.

```rust
processor.set_storage(Box::new(PlotModel::default()));
```
