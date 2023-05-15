import { type FunctionComponent, useState } from 'react';
import { Card } from '@mui/material';
import Iframe from 'react-iframe';
import { invoke } from '@tauri-apps/api/tauri';
import './index.scss';

interface Props {
  port: string;
  url: string | undefined;
  path: string;
}

const Chat: FunctionComponent<Props> = ({ port, url = '127.0.0.1', path }) => {
  return (
    <Card>
      <Iframe url={`http://${url}:${port}${path}`} />
    </Card>
  );
};

export default Chat;
