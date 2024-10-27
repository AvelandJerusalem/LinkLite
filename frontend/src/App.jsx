import { useState } from 'react';
import './App.css'

function Input() {
  const handleSubmit = (formData) => {
    formData.preventDefault();
    alert(formData);

  }

  return (
    <>
      <form onSubmit={handleSubmit}>
        <input name="URLInput" />
      </form>
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
