import * as _ from 'lodash';
import("../pkg/index.js").catch(console.error);

// ok += 'hi';

function component() {
    const element = document.createElement('div');

    element.innerHTML = _.join(['Hello', 'webpack'], ' ');

    return element;
}

document.body.appendChild(component());