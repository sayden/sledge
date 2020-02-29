# sledge

## Databases
### Reading
#### `GET /db/{db}/{id}[?channel={id}]`
Returns the record with the specified id.

##### Example 
* Request: `GET /db/my_db/mariete`
* Response
```json
{
  "error": false,
  "cause": null,
  "data": [
    {
      "id": "mariete",
      "name": "mario",
      "surname": "castro"
    }
  ]
}
```

#### `GET /db/{db}/_range/{id}`
Optional query: `?limit={usize}&skip={usize}&until_key={String}&end={String}&direction_forward={bool}?channel={String}`

Returns a feed of records starting from {id}.

#### `GET /db/{db}/_all`
Optional query params: `?limit={usize}&skip={usize}&until_key={String}&end={String}&direction_forward={bool}?channel={String}`

Returns a feed of records starting from the very beginning of database `{db}`.

### Writing
#### `PUT /db/{db}/{id}?channel={id}`
Inserts a new record with the provided id.

#### `PUT /db/{db}/_auto?channel={id}`
Inserts a new record generating a random v4 uuid.

#### `PUT /db/{db}?id={json_field}&channel={id}`
Inserts a new record in the database using the specified json field in the query as field. This operation is "slow" 
because the database will have to parse the incoming data as JSON before inserting. 

## Channels
### Read
#### `GET /channel/{id}`
Returns the specified channel by the {id}

#### `GET /channel/_all`
Returns all channels

### Write
#### `POST /channel/{id}`
Creates a new channel using the specified {id} in path.

#### `POST /channel/_auto`
Creates a new channel with a random v4 uuid.

## Stats
#### `GET /stats`