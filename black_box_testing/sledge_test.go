package sledge

import (
	"bytes"
	"encoding/json"
	"fmt"
	"github.com/stretchr/testify/assert"
	"io/ioutil"
	"net/http"
	"testing"
)

func doReq(t *testing.T, method string, reqS string, data string) string {
	req, _ := http.NewRequest(method, reqS, bytes.NewReader([]byte(data)))

	res, err := http.DefaultClient.Do(req)
	if err != nil {
		assert.NoError(t, err)
	}
	defer res.Body.Close()

	byt, err := ioutil.ReadAll(res.Body)
	assert.NoError(t, err)

	return string(byt)
}
func doReqNoBody(t *testing.T, method string, reqS string) string {
	req, _ := http.NewRequest(method, reqS, nil)

	res, err := http.DefaultClient.Do(req)
	if err != nil {
		assert.NoError(t, err)
	}
	defer res.Body.Close()

	byt, err := ioutil.ReadAll(res.Body)
	assert.NoError(t, err)

	return string(byt)
}

func doReqWithUnmarshal(t *testing.T, method string, reqS string, data string) (string, map[string]interface{}) {
	byt := doReq(t, method, reqS, data)

	var i map[string]interface{}
	err := json.Unmarshal([]byte(byt), &i)
	assert.NoError(t, err)

	return byt, i
}

var known_id = "hello_world"

func TestPut(t *testing.T) {

	t.Run("put single doc requests", func(t *testing.T) {
		t.Run("put doc with id in path", func(t *testing.T) {
			for i := 0; i < 100; i++ {
				dataToInsert := fmt.Sprintf(`{ "name": "name_%02d", "surname": "surname_%02d", "age": %d, "object": {"inner_1":"hello_world_%02d", "rating": %d}}`, i, i, i, i, i%5)
				byt := doReq(t, http.MethodPut, "http://localhost:3000/_db/test_db/_auto_time", dataToInsert)
				assert.Equal(t, `{"error":false,"cause":null,"data":null}`, string(byt))
			}
		})

		t.Run("put doc with id in json", func(t *testing.T) {
			byt := doReq(t, http.MethodPut, "http://localhost:3000/_db/test_db?id_path=hello", `{"hello":"world"}`)
			assert.Equal(t, `{"result":{"error":false,"cause":null,"db":"test_db"},"id":"world"}`, string(byt))
		})

		t.Run("put doc with _auto id", func(t *testing.T) {
			s, i := doReqWithUnmarshal(t, http.MethodPut, "http://localhost:3000/_db/test_db/_auto", `{"hello":"world"}`)
			ok := i["result"].(map[string]interface{})["error"].(bool)
			assert.False(t, ok, s)
		})

		t.Run("put doc without id", func(t *testing.T) {
			s, i := doReqWithUnmarshal(t, http.MethodPut, "http://localhost:3000/_db/test_db", `{"hello":"world"}`)
			no_ok := i["result"].(map[string]interface{})["error"].(bool)
			assert.True(t, no_ok, s)
		})
	})
}

func TestGet(t *testing.T) {
	t.Run("get single doc requests", func(t *testing.T) {
		t.Run("get doc by id in path", func(t *testing.T) {
			s := doReqNoBody(t, http.MethodGet, "http://localhost:3000/_db/test_db/"+known_id+"_1")
			assert.Contains(t, s, `"id":"hello_world_1"}`, s)
		})

		t.Run("try to get doc without id", func(t *testing.T) {
			s := doReqNoBody(t, http.MethodGet, "http://localhost:3000/_db/test_db?id_path="+known_id)
			assert.Contains(t, s, `{"result":{"error":true`)
		})

		t.Run("get _all docs", func(t *testing.T) {
			s := doReqNoBody(t, http.MethodGet, "http://localhost:3000/_db/test_db/_all")
			assert.Contains(t, s, `"id":"world",`)
			assert.Contains(t, s, `"hello":"world"`)
			assert.Contains(t, s, `"hello":"world"`)
		})
	})
}

func TestPost(t *testing.T) {
	t.Run("get single doc requests", func(t *testing.T) {
		t.Run("get doc by id in path using a POST channel", func(t *testing.T) {
			s := doReq(t, http.MethodPost, "http://localhost:3000/_db/test_db/hello_world_1", `{"name":"mario","channel":[{"type":"join","field":["name","surname"],"separator":" ","new_field":"full_name"}]}`)
			assert.Contains(t, s, `"full_name":"mario castro"`)
		})
	})
	t.Run("get _all doc requests", func(t *testing.T) {
		s := doReq(t, http.MethodPost, "http://localhost:3000/_db/test_db/_all", `{"name":"mario","channel":[{"type":"join","field":["name","surname"],"separator":" ","new_field":"full_name"}]}`)
		assert.Contains(t, s, `"full_name":"mario castro"`)
		assert.Contains(t, s, `"hello_world_1"`)
		assert.Contains(t, s, `"hello_world_2"`)
		assert.Contains(t, s, `"hello_world_3"`)
	})
}

func TestChannels(t *testing.T) {

}
