import React, { Component } from 'react';
import './App.css';

import SearchField from './SearchField.js';
import ImageSlot from './ImageSlot.js';
import ImageEditor from './ImageEditor.js'
import TagList from './TagList.js'
import TifariAPI from "./APIComms.js"

class App extends Component {

    constructor(props) {
        super(props);

        this.state = {
            queriedImages: [],
            queriedImagesAsElements: [],
            isInToTagList: false,
            selectedImages: [],
            displayTagList: false,
            tagQueueSize: 0,
            searchTags: [],
        };

        this.onSearch           = this.onSearch.bind(this);
        this.requeryImages      = this.requeryImages.bind(this);
        this.onSelectImage      = this.onSelectImage.bind(this);
        this.hideEditorSidebar  = this.hideEditorSidebar.bind(this);
        this.escKeyListener     = this.escKeyListener.bind(this);
        this.viewToBeTaggedList = this.viewToBeTaggedList.bind(this);
        this.mapSelectedImages  = this.mapSelectedImages.bind(this);
        this.onEditorAddTag     = this.onEditorAddTag.bind(this);
        this.onEditorRemoveTag  = this.onEditorRemoveTag.bind(this);
        this.queryTagListSize   = this.queryTagListSize.bind(this);
        this.inStateMapSelectedImages  = this.inStateMapSelectedImages.bind(this);
        this.inStateRemoveImageFromQueryIfTagsArentInQuery = this.inStateRemoveImageFromQueryIfTagsArentInQuery.bind(this);
    }

    hideEditorSidebar() {
        this.setState({
            selectedImages: []
        });
    }

    escKeyListener(ev) {
        if(ev.key === "Escape") {
            this.hideEditorSidebar();
        }
    }

    componentWillMount() {
        this.queryTagListSize()
    }

    componentDidMount() {
        document.addEventListener("keypress", this.escKeyListener, false);
    }

    componentWillUnmount() {
        document.removeEventListener("keypress", this.escKeyListener, false);
    }

    // callback that's called when we want to select an image
    onSelectImage(img) {
        // avoid duplicate images
        for(var i = 0; i < this.state.selectedImages.length; i++) {
            if(img.id === this.state.selectedImages[i].id) {
                return;
            }
        }

        // append im to selectedImages
        this.setState(prevState => ({
          selectedImages: [...prevState.selectedImages, img]
        }));
    }

    // maps a given image array to image slots for display in tghe main page
    inStateMapSelectedImages(images) {
        let mappedImages = images.map(img => 
            <ImageSlot 
                img={img} 
                key={img.id} 
                onClick={() => this.onSelectImage(img)}
            />
        );

        return {
            queriedImages: images.results,
            queriedImagesAsElements: mappedImages
        };
    }

    mapSelectedImages(images) {
        let data = this.inStateMapSelectedImages(images);
        this.setState({
            queriedImages: data.queriedImages,
            queriedImagesAsElements: data.queriedImagesAsElements,
        });
    }

    // will display search results in the main page
    requeryImages(tagsArray) {
        this.setState({
            searchTags: tagsArray,
            isInToTagList: false,
        });

        TifariAPI.search(tagsArray).then(this.mapSelectedImages);
    }

    // will display the to be tagged list in the main page
    viewToBeTaggedList() {
        this.setState({
            searchTags: [],
            isInToTagList: true,
        });

        TifariAPI.getToBeTaggedList().then(this.mapSelectedImages);
    }

    // callback that's called when we want to search the backend for tags
    onSearch(query) {
        let tags = query.split(" ");
        this.requeryImages(tags);
    }

    // updates the tagQueueSize state from the backend
    queryTagListSize() {
        TifariAPI.getTagQueueSize()
            .then(size => {
                this.setState({
                    tagQueueSize: size
                });
            });
    }

    // setState mutation helper method
    inStateRemoveImageFromQueryIfTagsArentInQuery(state, img, tags) {

        let retval = { 
            queriedImages: state.queriedImages
        };

        // remove image from search if the intersection between
        // image tags and search tags is empty
        const intersection = state.searchTags.filter(val => -1 !== tags.indexOf(val));

        if(0 >= intersection.length) {
            console.log("should erase");

            // TODO : errs here
            let images = state.queriedImages;
            let imgIndex = images.findIndex(i => i.id === img.id);

            if(imgIndex !== -1) {
                images.splice(imgIndex, 1);
                // update listed images and merge with retval.
                Object.assign(retval, this.inStateMapSelectedImages(state.queriedImages));
            }
        }

        return retval;
    }

    // callback that's called when we remove a tag from an image
    onEditorRemoveTag(image, tag) {
        TifariAPI.removeTags([tag.id], [image.id]);
        
        this.setState(oldState => {
            let images = oldState.selectedImages;

            let imgIndex = images.findIndex(i => i.id === image.id);
            if(imgIndex === -1) 
                return {};
            
            let tags = oldState.selectedImages[imgIndex].tags;
            let tagIndex = tags.findIndex(t => t.id === tag.id);
            if(tagIndex === -1) 
                return {};

            // remove tag from img tags array
            tags.splice(tagIndex, 1);
            oldState.queriedImages = this.inStateRemoveImageFromQueryIfTagsArentInQuery(oldState, image, tags);
            return {
                queriedImages: oldState.queriedImages,
                selectedImages: oldState.selectedImages,
            };
        });


        // TODO : update tag list

        this.queryTagListSize();
    }

    // callback that's called whenever we add a tag to an image
    onEditorAddTag(index, tags) {
        this.setState(oldState => {
            let image = oldState.selectedImages[index];
            let newArray = image.tags.concat(tags);
            image.tags = newArray;
    
            if(oldState.isInToTagList)
                this.inStateRemoveImageFromQueryIfTagsArentInQuery(oldState, image, image.tags);

            return oldState;
        });
        
        this.queryTagListSize();

        // TODO : update tag list
    }

    render() {
        return (
            <div className="App">
                {this.state.selectedImages.length > 0 &&
                    <ImageEditor 
                        images={this.state.selectedImages}
                        addTagsToImage={this.onEditorAddTag}
                        onRemoveTag={this.onEditorRemoveTag}
                    />
                }

                {this.state.displayTagList &&
                    <TagList
                    />
                }

                <header>
                    <button onClick={this.viewToBeTaggedList}>View To-Tag List({this.state.tagQueueSize})</button>
                    <button onClick={() => TifariAPI.reloadRoot()}>Reload images</button>
            
                    <button 
                        onClick={() => this.setState({displayTagList: !this.state.displayTagList})}
                        >
                        {this.state.displayTagList ? "Hide" : "Show"} tag list
                    </button>

                    <SearchField onChange = {this.onSearch} />
                </header>

                <ul>{this.state.queriedImagesAsElements}</ul>
            </div>
        );
    }
}

export default App;
