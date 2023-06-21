import { useRef, useState } from 'react';
import {
  Input,
  Button,
  IconButton,
  FormControl,
  Card,
  CardContent,
} from '@mui/material';
import FileOpenIcon from '@mui/icons-material/FileOpen';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import Chat from '../Chat';
import styles from './index.module.scss';

function App() {
  const [command, setCommand] = useState('');
  const btnRef = useRef<HTMLButtonElement>(null);

  return (
    <div className={styles.container}>
      <Chat
        url="127.0.0.1"
        port="8080"
        path="/twitch/v2/index.html?channel=maybejules&size=3&font=0&stroke=0&shadow=0&fade=30"
      />

      <Card>
        <CardContent>
          <FormControl fullWidth>
            <Input
              placeholder="Enter command"
              value={command}
              onKeyDown={(key) => {
                if (key.key === 'Enter') {
                  btnRef.current?.click();
                }
              }}
              onChange={(e) => setCommand(e.target.value)}
              style={{ gridArea: 'b' }}
            />

            <Button
              ref={btnRef}
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
              style={{ gridArea: 'c' }}
            >
              Send Command
            </Button>

            <IconButton
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

                // TODO: Handle the selected file
              }}
            >
              <FileOpenIcon></FileOpenIcon>
            </IconButton>
          </FormControl>
        </CardContent>
      </Card>
    </div>
  );
}

export default App;
