import { useState } from 'react';
import validator from 'validator';
import axios from 'axios';
import './App.css'

function Input() {
  const [err, setErr] = useState();
  const [url, setUrl] = useState();
  const apiURL = 'https://short.jeremypetch.com'


  const handleSubmit = async (e) => {
    e.preventDefault();
    const form = e.target;
    const formData = new FormData(form);
    const urlIn = formData.get("URLInput");
    if (!validator.isURL(urlIn)) {
      setErr("Invalid URL");
    } else {
      setErr("");
      setUrl("");
      const headers = { 'Content-Type': 'application/json', 'Access-Control-Allow-Origin': '*' };
      const res = await axios.post(apiURL, { url: urlIn }, headers)
      if (res.status != 200) {
        setErr("Failed to post - " + res.data);
      } else {
        setUrl(res.data);
      }

    }

  }

  return (
    <>
      <form method="post" onSubmit={handleSubmit}>
        <input name="URLInput" />
        <br />
        <button type="submit">Submit</button>
      </form>
      {url != '' && <div>Shortened URL: {url}</div>}
      <div color='red'>{err}</div>
    </>
  )

}

function App() {
  return (
    <>
      <h1>LinkLite</h1>
      <h2>A URL shortening tool</h2>
      <hr />
      <Input />
    </>
  )
}

export default App
