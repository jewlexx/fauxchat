import { useRef, useState } from 'react';
import { Card, CardContent, TextField, Button } from '@mui/material';
import { FaFileImport } from 'react-icons/fa';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import Chat from '../Chat';
import styles from './index.module.scss';

// TODO: Fix badges

function handleSelected(selected: string | null): Promise<string> {
  // Handle the selected file by calling the tauri load_file command
  return invoke('load_file', { path: selected });
}

function App() {
  const [command, setCommand] = useState('');
  const [error, setError] = useState<string | null>(null);
  const btnRef = useRef<HTMLButtonElement>(null);

  return (
    <div className={styles.container}>
      <Chat
        className={styles.chatContainer}
        url="127.0.0.1"
        port="8080"
        path="/twitch/v2/index.html?channel=possiblyjuliette&size=3&font=0&stroke=0&shadow=0"
      />

      <Card className={styles.controls}>
        <CardContent>
          <TextField
            error={error !== null}
            helperText={error}
            placeholder="Enter command"
            value={command}
            onKeyDown={(key) => {
              if (key.key === 'Enter') {
                btnRef.current?.click();
              }
            }}
            onChange={(e) => {
              setError(null);
              setCommand(e.target.value);
            }}
          />
          <Button
            ref={btnRef}
            onClick={() => {
              const oldCommand = command;

              invoke('invoke_command', { command })
                .then(() => {
                  // Only reset if the user has not updated the command
                  if (oldCommand === command) {
                    setCommand('');
                  }
                })
                .catch((error) => {
                  setError(error);
                });
            }}
          >
            Send Command
          </Button>

          <Button
            onClick={async () => {
              const selected = await open({
                multiple: false,
                filters: [
                  {
                    name: 'Commands',
                    extensions: ['commands'],
                  },
                ],
              });

              if (typeof selected === 'string') {
                handleSelected(selected).catch(setError);
              }
            }}
          >
            <FaFileImport></FaFileImport>
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}

export default App;
