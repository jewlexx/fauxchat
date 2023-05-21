import { type FunctionComponent } from 'react';
import { Card, CardContent } from '@mui/material';
import Iframe from 'react-iframe';
import styles from './index.module.scss';

interface Props {
  port: string;
  url: string | undefined;
  path: string;
}

const Chat: FunctionComponent<Props> = ({ port, url = '127.0.0.1', path }) => {
  return (
    <Card sx={{ marginBottom: '1rem' }}>
      <Iframe url={`http://${url}:${port}${path}`} className={styles.frame} />
    </Card>
  );
};

export default Chat;
