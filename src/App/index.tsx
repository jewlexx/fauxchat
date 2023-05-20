import { useState } from 'react';
import { Input, Select, Option, Button } from '@mui/joy';
import { invoke } from '@tauri-apps/api/tauri';
import Chat from '../Chat';
import styles from './index.module.scss';

enum Command {
  Send,
  Sleep,
}

function App() {
  const [message, setCommand] = useState('');

  return (
    <div className={styles.container}>
      <Chat
        url="127.0.0.1"
        port="8080"
        path="/twitch/v2/index.html?channel=maybejules&size=3&font=0&stroke=0&shadow=0&fade=30"
      />

      <Button onClick={() => {}} style={{ gridArea: 'e' }}></Button>
    </div>
  );
}

export default App;
