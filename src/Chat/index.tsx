import { type FunctionComponent } from 'react';
import Iframe from 'react-iframe';

interface Props {
  className?: string;
  url?: string;
  port: string;
  path: string;
}

const Chat: FunctionComponent<Props> = ({
  className,
  port,
  url = '127.0.0.1',
  path,
}) => {
  return (
    <div className={className}>
      <Iframe url={`http://${url}:${port}${path}`} />
    </div>
  );
};

export default Chat;
