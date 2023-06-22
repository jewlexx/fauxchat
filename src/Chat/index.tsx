import { type FunctionComponent } from 'react';
import Iframe from 'react-iframe';
import styles from './index.module.scss';

interface Props {
  port: string;
  url: string | undefined;
  path: string;
}

const Chat: FunctionComponent<Props> = ({ port, url = '127.0.0.1', path }) => {
  return (
    <Iframe url={`http://${url}:${port}${path}`} className={styles.frame} />
  );
};

export default Chat;
