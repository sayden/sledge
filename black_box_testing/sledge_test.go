package sledge

import (
	"bytes"
	"encoding/json"
	"github.com/stretchr/testify/assert"
	"io/ioutil"
	"net/http"
	"testing"
)

func TestWrite(t *testing.T) {
	doReq := func(reqS string, data string) string {
		req, _ := http.NewRequest(http.MethodPut, reqS, bytes.NewReader([]byte(data)))

		res, err := http.DefaultClient.Do(req)
		if err != nil {
			assert.NoError(t, err)
		}
		defer res.Body.Close()

		byt, err := ioutil.ReadAll(res.Body)
		assert.NoError(t, err)

		return string(byt)
	}

	doReqWithUnmarshal := func(reqS string, data string) (string, map[string]interface{}) {
		byt := doReq(reqS, data)

		var i map[string]interface{}
		err := json.Unmarshal([]byte(byt), &i)
		assert.NoError(t, err)

		return byt, i
	}

	t.Run("put single doc requests", func(t *testing.T) {
		t.Run("put doc with id in path", func(t *testing.T) {
			byt := doReq("http://localhost:3000/db/other_db/hello_world", "hello world")
			assert.Equal(t, `{"result":{"error":false,"cause":null,"db":"other_db"},"id":"hello_world"}`, string(byt))
		})

		t.Run("put doc with id in json", func(t *testing.T) {
			byt := doReq("http://localhost:3000/db/other_db?id=hello", `{"hello":"world"}`)
			assert.Equal(t, `{"result":{"error":false,"cause":null,"db":"other_db"},"id":"world"}`, string(byt))
		})

		t.Run("put doc with _auto id", func(t *testing.T) {
			s, i := doReqWithUnmarshal("http://localhost:3000/db/other_db/_auto", `{"hello":"world"}`)
			ok := i["result"].(map[string]interface{})["error"].(bool)
			assert.False(t, ok, s)
		})

		t.Run("put doc without id", func(t *testing.T) {
			s, i := doReqWithUnmarshal("http://localhost:3000/db/other_db", `{"hello":"world"}`)
			no_ok := i["result"].(map[string]interface{})["error"].(bool)
			assert.True(t, no_ok, s)
		})
	})
}

func TestRead(t *testing.T) {
	t.Run("get single doc requests", func(t *testing.T) {
		t.Run("get doc by id in path", func(t *testing.T) {

		})
	})
}
