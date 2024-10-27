import './App.css'

function Input() {
  function handleSubmit(e) {
    // Stops browser from reloading page
    e.preventDefault();

    // Read form data
    const form = e.target;
    const formData = new FormData(form);
  }
 
  return (
    <><form method="post" onSubmit={handleSubmit}>
      <label>
        URL: <input name="URLInput" />
      </label>
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
