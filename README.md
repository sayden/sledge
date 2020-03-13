# sledge

## Read queries
* [*] All documents in a db, forward direction `/_db/{db}/_all`
* [*] All documents in a db, reverse direction `/_db/{db}/_all_reverse`
* [*] Single doc in db `/_db/{db}/{id}`
* [*] Range of docs in db since an id`/_db/{db}/_since/{id}`
* [*] Docs prefixed with `/_db/{db}/{id}*`
* [*] Get list of all dbs

### Options
* [*] Include id in response
* [*] Limit docs
* [*] Infinite until finding key
* [*] Only field matching key in json
* [*] Infinite until finding key in json
* [*] Skip N first records
* [*] Mutate results by specifying an already stored mutator channel id
* [ ] Read to output
* [ ] SQL that covers SELECT _____ FROM ______ WHERE ______;
    * [*] Simple `SELECT [field]` and `SELECT *`
    * [*] Projections over fields (no functions)
    * [*] WHERE binary clauses for direct fields like `SELECT * FROM db WHERE age > 30 and name = 'mario'`
    * [*] WHERE binary compound clauses like `(a OR b) AND c`
    * [*] LIMIT expression
    * [ ] SKIP like expression (FETCH? OFFSET?)

## Write queries
* [*] Write single doc
* [ ] Write batch of docs separated by newline

### Options
* [*] Mutate results by specifying an already stored mutator channel id
* [*] Get the id from inside the JSON
* [*] Auto-generate an id
* [*] Auto-generate a time based id (insertion time)
* [ ] Write from input

## Delete queries

* [ ] Delete single value

## Other

* [ ] Enforce JSON data
* [ ] Secondary indices
* [ ] Outputs
  * [ ] HTTP
  * [ ] Kafka
  * [ ] NATS
* [ ] Inputs
  * [ ] Kafka
  * [ ] NATS
* [ ] Script mutator
* [ ] DB Statistics
* [ ] Keep alive for range queries
* [ ] Tail -f queries
