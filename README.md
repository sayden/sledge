# sledge

## Read queries
* [*] All documents in a db, forward direction `/_db/{db}/_all`
* [*] All documents in a db, reverse direction `/_db/{db}/_all_reverse`

* [*] Single doc in db `/_db/{db}/{id}`
* [*] Range of docs in db since an id`/_db/{db}/_since/{id}`

* [*] Docs prefixed with `/_db/{db}/{id}*`

### Options
* [*] Include id in response
* [ ] Limit docs
* [ ] Infinite until finding key
* [ ] Infinite until finding key in json
* [ ] Skip N first records
* [ ] Mutate results by specifying an already stored mutator channel id
* [ ] Read to output

## Write queries
* [ ] Write single doc `/_db/{db}/[{id} / _auto /][?field_path]`

### Options
* [ ] Mutate before write specifying an already stored channel id
* [ ] Get the id from inside the JSON
* [ ] Auto-generate an id
* [ ] Write from input

## Other

* [ ] Enforce JSON data
* [ ] Secondary indices
* [ ] Outputs
  * [ ] HTTP
* [ ] Inputs
  * [ ] Kafka
* [ ] Script mutator
* [ ] DB Statistics
* [ ] Keep alive for range queries
