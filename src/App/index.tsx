import { useState } from 'react';
import { Input, Select, Option, Button } from '@mui/joy';
import { invoke } from '@tauri-apps/api/tauri';
import Chat from '../Chat';
import styles from './index.module.scss';

function App() {
  const [command, setCommand] = useState('');

  return (
    <div className={styles.container}>
      <Chat
        url="127.0.0.1"
        port="8080"
        path="/twitch/v2/index.html?channel=maybejules&size=3&font=0&stroke=0&shadow=0&fade=30"
      />

      <Input
        placeholder="Enter command"
        value={command}
        onChange={(e) => setCommand(e.target.value)}
      ></Input>

      <Button
        onClick={() => {
          const oldCommand = command;

          const p = invoke('invoke_command', { command });
          p.then((_) => {
            // Only reset if the user has not updated the command
            if (oldCommand === command) {
              setCommand('');
            }
          });
        }}
        style={{ gridArea: 'e' }}
      >
        Send Command
      </Button>
    </div>
  );
}

export default App;
