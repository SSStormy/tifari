import React, { Component } from 'react';
import TifariAPI from "./APIComms.js"

class ImageField extends Component {
    render() {
        return (
            <span 
                key={this.props.img.id}
                onClick={this.props.onClick}>
                <img 
                    alt=""
                    src={TifariAPI.getImageUrl(this.props.img)}
                    style={{maxWidth: "512px"}}
                />
            </span>
        );
    }
}

export default ImageField;
