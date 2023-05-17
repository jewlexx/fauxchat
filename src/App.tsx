import { useState } from 'react';
import { Input, Select, Option, Button } from '@mui/joy';
import { invoke } from '@tauri-apps/api/tauri';
import Chat from './Chat';
import styles from './App.module.scss';

enum Command {
  Send,
  Sleep,
}

function App() {
  const [message, setMessage] = useState('');
  const [count, setCount] = useState(0);
  const [delay, setDelay] = useState(0);
  const [command, setCommand] = useState<Command | null>(null);

  // TODO: User Picker

  return (
    <div className={styles.container}>
      <Chat
        url="127.0.0.1"
        port="8080"
        path="/twitch/v2/index.html?channel=maybejules&size=3&font=0&stroke=0&shadow=0&fade=30"
      />
      <Select
        placeholder="Choose command"
        value={command}
        onChange={(_, newValue) => setCommand(newValue)}
      >
        <Option value={Command.Send}>Send</Option>
        <Option value={Command.Sleep}>Sleep</Option>
      </Select>
      {command === Command.Send && (
        <>
          <Input value={message} onChange={(e) => setMessage(e.target.value)} />

          <Input
            type="number"
            placeholder="Count"
            value={count}
            onChange={(e) => setCount(parseInt(e.target.value, 10))}
          />
        </>
      )}

      <Input
        type="number"
        placeholder="Delay"
        value={count}
        onChange={(e) => setDelay(parseInt(e.target.value, 10))}
      />

      <Button
        onClick={() => {
          if (command == Command.Sleep) {
            throw {
              name: 'NotImplementedError',
              message: 'sleep command is not implemented yet',
            };
          }
          const p = invoke('send_message', {
            username: 'random',
            message,
            count,
            delay,
          });

          p.then(() => {
            setMessage('');
            setCount(0);
          });
        }}
      >
        Send Message
      </Button>
    </div>
  );
}

export default App;
