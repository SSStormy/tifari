import React, { Component } from 'react';
import './ImageEditor.css';
import Constants from './Config.js';

class TagRemoveButton extends Component {
    render() {
        return (
            <button>{this.props.tag.name}</button>
        );
    }
}

class ImageEditor extends Component {
    render() {

        const tags = this.props.img.tags.map(t => <TagRemoveButton key={t.id} tag={t}/>);

        return (
            <div className="ImageEditor_sidebar">
                <img
                    alt = ""
                    src = {Constants.ENDPOINT_API_IMAGE + this.props.img.path}
                />
                <ul>{tags}</ul>
            </div>
        );
    }
}

export default ImageEditor;
