import React from 'react';
import ReactDOM from 'react-dom';
import Bootstrapper from './App';
import registerServiceWorker from './registerServiceWorker';

ReactDOM.render(<Bootstrapper/>, document.getElementById('root'));
registerServiceWorker();
