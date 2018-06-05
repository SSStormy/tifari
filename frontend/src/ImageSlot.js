import React, { Component } from 'react';
import Constants from './Config.js'

class ImageField extends Component {
    render() {
        return (
            <span 
                key={this.props.img.id}
                onClick={this.props.onClick}>
                <img 
                    alt=""
                    src={Constants.ENDPOINT_API_IMAGE + this.props.img.path}
                    style={{maxWidth: "512px"}}
                />
            </span>
        );
    }
}

export default ImageField;
