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
  const [message, setMessage] = useState('');
  const [count, setCount] = useState(0);
  const [command, setCommand] = useState<Command | null>(null);

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
        style={{ gridArea: 'b' }}
      >
        <Option value={Command.Send}>Send</Option>
        <Option value={Command.Sleep}>Sleep</Option>
      </Select>
      {command === Command.Send && (
        <Input
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          style={{ gridArea: 'c' }}
        />
      )}
      <Input
        type="number"
        value={count}
        onChange={(e) => setCount(parseInt(e.target.value, 10))}
        style={{ gridArea: 'd' }}
      />

      <Button
        placeholder="Send Message"
        onClick={() => {
          throw {
            name: 'NotImplementedError',
            message: 'too lazy to implement',
          };
        }}
        style={{ gridArea: 'e' }}
      ></Button>
    </div>
  );
}

export default App;
