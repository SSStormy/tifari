import React, { Component } from 'react';
import './ImageEditor.css';
import TifariAPI from "./APIComms.js"

class TagRemoveButton extends Component {
    render() {
        return (
            <button>{this.props.tag.name}</button>
        );
    }
}

class ImageEditor extends Component {

    constructor(props) {
        super(props);

        this.onTagAddInputChange = this.onTagAddInputChange.bind(this);
    }

    onTagAddInputChange(ev) {
        if(ev.key === "Enter") {
        }
    }

    render() {

        const tags = this.props.img.tags.map(t => <TagRemoveButton key={t.id} tag={t}/>);

        return (
            <div className="ImageEditor_sidebar">
                <img
                    alt = ""
                    src = {TifariAPI.getImageUrl(this.props.img)}
                />

                <ul>{tags}</ul>

                <div>
                    <input type="text" 
                        onChange={this.onTagAddInputChange}
                    />
                </div>
            </div>
        );
    }
}

export default ImageEditor;
