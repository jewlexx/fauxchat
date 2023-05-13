import { useState } from 'react';
import reactLogo from './assets/react.svg';
import { invoke } from '@tauri-apps/api/tauri';
import Chat from './Chat';
import './App.scss';

function App() {
  const [greetMsg, setGreetMsg] = useState('');
  const [name, setName] = useState('');

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setGreetMsg(await invoke('greet', { name }));
  }

  return (
    <div className="container">
      <Chat port="8080" url="127.0.0.1" path="/twitch/index.html" />
    </div>
  );
}

export default App;
