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

        this.state = {
            tagString: "",
        }

        this.onTagAddInputChange = this.onTagAddInputChange.bind(this);
        this.submitTags = this.submitTags(this);
    }

    submitTags() {

    }

    onTagAddInputChange(ev) {
        this.setState({tagString: ev.target.value.trim()});
        if(ev.key === "Enter") {
            this.submitTags();
        }
    }

    render() {
        const tags = this.props.images.map(img => {
            let set = new Set();
                
            for(var i = 0; i < img.tags.length; i++) {
                set.add(img.tags[i]);
            }

            return Array.from(set).map(t => <TagRemoveButton key={t.id} tag={t}/>);
        });

        const imgs = this.props.images.map(img => 
            <img
                key = {img.id}
                alt = {img.path}
                src = {TifariAPI.getImageUrl(img)}
            />
        );

        return (
            <div className="ImageEditor_sidebar">

                <div>
                    <ul>{imgs}</ul>
                </div>

                <div>
                    <ul>{tags}</ul>
                </div>

                <div>
                    <input type="text" 
                        onChange={this.onTagAddInputChange}
                    />
                </div>
                
                {this.state.tagString.length > 0 &&
                    <button>Add tags</button>
                }
            </div>
        );
    }
}

export default ImageEditor;
