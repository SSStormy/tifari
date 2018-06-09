import React, { Component } from 'react';
import './App.css';

import ImageEditor from './ImageEditor.js';
import {TagList, defaultTagOrdering} from './TagList.js';
import TifariAPI from "./APIComms.js";
import {ldebug, assert} from "./Logging.js";

import CssBaseline from '@material-ui/core/CssBaseline';
import Icon from '@material-ui/core/Icon';
import Snackbar from '@material-ui/core/Snackbar';
import Grid from '@material-ui/core/Grid';
import Button from '@material-ui/core/Button';
import AppBar from '@material-ui/core/AppBar';
import Toolbar from '@material-ui/core/Toolbar';
import TextField from '@material-ui/core/TextField';
import Card from '@material-ui/core/Card';
import CardMedia from '@material-ui/core/CardMedia';
import CardContent from '@material-ui/core/CardContent';
import CardActions from '@material-ui/core/CardActions';
import Typography from '@material-ui/core/Typography';
import Paper from '@material-ui/core/Paper';
import GridListTile from '@material-ui/core/GridListTile';
import GridList from '@material-ui/core/GridList';
import ButtonBase from '@material-ui/core/ButtonBase';

class StateMutator {
    constructor(app, oldState) {
        this.newState = {};

        this.oldState = oldState;
        this.app = app;
    }

    getFinalState() {
        return this.newState;
    }

    getOldState() {
        return this.oldState;
    }

    setTags(tags) {
        ldebug("Setting tags list to");
        ldebug(tags);

        this.newState.tags = tags;
        return this;
    }

    setTagOrdering(ordering) {
        this.newState.tagOrdering = ordering;
        return this;
    }

    showSnackbar(msg) {
        this.newState.showSnackbar = true;
        this.newState.snackbarMessage = msg;

        return this;
    }

    hideSnackbar() {
        this.newState.showSnackbar = false;

        return this;
    }

    orderTags() {
        
        if(!this.newState.hasOwnProperty("tags")) {
            this.newState.tags = this.oldState.tags;
        }

        let tagOrdering = this.newState.hasOwnProperty("tagOrdering")
            ? this.newState.tagOrdering
            : this.oldState.tagOrdering;

        tagOrdering.order(this.newState.tags);
    }

    setImageList(images) {
        ldebug("Setting image list.");
        ldebug(images);

        this.newState.queriedImages = images;
        return this;
    }

    addImageToList(image) {
        ldebug("Adding image to list");
        ldebug(image);

        let images = this.getQueriedImagesAndMarkMutated();

        // avoid duplicate images
        if(images.findIndex(i => i.id === image.id) !== -1)
            return this;
    
        images.push(image);

        return this;
    }

    getQueriedImagesAndMarkMutated() {
        if(!this.newState.hasOwnProperty("queriedImages")) {
            // if we copy this, we're going to have to add a second layer of
            // indirection by making this queriedImage array store
            // references to cached values within a cachce area somewhere
            this.newState.queriedImages = this.oldState.queriedImages;
        }

        return this.newState.queriedImages;
    }

    // doesn't update the image list.
    removeImageFromList(image) {
        ldebug("Removing image from list.");
        ldebug(image);

        let images = this.getQueriedImagesAndMarkMutated();
    
        let imgIndex = images.findIndex(i => i.id === image.id);
        if(imgIndex === -1) return this;

        images.splice(imgIndex, 1);

        return this;
    }

    // doesn't update the image list.
    removeTagFromImage(image, tag) {
        ldebug("Removing tag from image");
        ldebug(image);
        ldebug(tag);

        let tagIndex = image.tags.findIndex(t => t.id === tag.id);
        if(tagIndex === -1) return this;

        image.tags.splice(tagIndex, 1);

        return this;
    }

    // doesn't update the image list.
    // image must be a part of the app state
    addTagToImage(image, tag) {
        ldebug("Adding tag to image");
        ldebug(image);
        ldebug(tag);

        let tagIndex = image.tags.findIndex(t => t.id === tag.id);

        if(tagIndex === -1) {
            image.tags.push(tag);
        }

        return this;
    }

    getSelectedImages() {
        if(!this.newState.hasOwnProperty("selectedImages")) {
            ldebug("Marked selected images as dirty.");
            this.newState.selectedImages = this.oldState.selectedImages;
        }

        return this.newState.selectedImages;

    }

    addSelectedImage(image) {
        ldebug("Adding selected image");
        ldebug(image);

        let images = this.getSelectedImages();

        // avoid duplicate images
        if(images.findIndex(i => i.id === image.id) !== -1)
            return this;

        images.push(image);
        return this;
    }

    removeSelectedImage(image) {
        ldebug("Removing selected image");
        ldebug(image);

        let images = this.getSelectedImages();
        let imgIndex = images.findIndex(i => i.id == image.id);

        if(imgIndex === -1) 
            return this;
        
        images.splice(imgIndex, 1);

        return this;
    }

    clearSelectedImages() {
        ldebug("Clearing selected images");

        this.newState.selectedImages = [];
        return this;
    }

    setSearchTags(tagsArray) {
        ldebug("Setting search tags");
        ldebug(tagsArray);

        this.newState.searchTagNames = tagsArray;
        return this;
    }

    setIsInToBeTaggedList(state) {
        ldebug("Setting is in to be tagged list state to");
        ldebug(state);

        this.newState.isInToTagList = state;
        return this;
    }

    setTagListDisplayState(state) {
        ldebug("Setting tag list display state to");
        ldebug(state);

        this.newState.displayTagList = state;
        return this;
    }

    setToBeTaggedListSize(size) {
        ldebug("Seting to be tagged list size to");
        ldebug(size);

        this.newState.tagQueueSize = size;
        return this;
    }
}

class App extends Component {

    constructor(props) {
        super(props);

        this.state = {
            queriedImages: [],
            isInToTagList: false,
            selectedImages: [],
            displayTagList: false,
            tagQueueSize: 0,
            searchTagNames: [],
            tags: [],
            tagOrdering: defaultTagOrdering,
        };

        this.refSearchBar = React.createRef();

        this.removeImageFromSelected        = this.removeImageFromSelected.bind(this);
        this.foreignSetTagListOrdering      = this.foreignSetTagListOrdering.bind(this);
        this.foreignToggleTagListDisplay    = this.foreignToggleTagListDisplay.bind(this);
        this.foreignEscKeyListener          = this.foreignEscKeyListener.bind(this);
        this.foreignViewToBeTaggedList      = this.foreignViewToBeTaggedList.bind(this);

        this.foreignOnEditorRemoveTagFromSelected = this.foreignOnEditorRemoveTagFromSelected.bind(this);
        this.foreignOnEditorAddTagToSelected = this.foreignOnEditorAddTagToSelected.bind(this);
        this.foreignAddTagToSearch = this.foreignAddTagToSearch.bind(this);
    }

    hideEditorSidebar() {
        this.mutateState(mut => mut.clearSelectedImages());
    }

    foreignEscKeyListener(ev) {
        if(ev.key === "Escape") {
            this.hideEditorSidebar();
        }
    }

    componentWillMount() {
        this.updateToBeTaggedListSize();
        this.updateTagList();
    }

    componentDidMount() {
        document.addEventListener("keypress", this.foreignEscKeyListener, false);
    }

    componentWillUnmount() {
        document.removeEventListener("keypress", this.foreignEscKeyListener, false);
    }

    mutateState(lambda) {
        this.setState(oldState => {
            let mutator = new StateMutator(this, oldState);
            lambda(mutator);
            return mutator.getFinalState();
        });
    }

    // callback that's called when we want to select an image
    onSelectImage(img) {
        this.mutateState(mut => {
            mut.addSelectedImage(img);
        })
    }

    // callback that's called when we want to search the backend for tags
    doImageSearch(query) {
        let tags = query.split(" ");

        TifariAPI.search(tags)
            .then(images =>
                this.mutateState(mut =>
                    mut.setSearchTags(tags)
                       .setImageList(images)
                       .setIsInToBeTaggedList(false))
                );
    }

    updateToBeTaggedListSize() {
        TifariAPI.getTagQueueSize().then(size => this.mutateState(mut => mut.setToBeTaggedListSize(size)));
    }

    foreignSetTagListOrdering(ordering) {
        this.mutateState(mut => 
            mut.setTagOrdering(ordering)
               .orderTags()
        );
    }

    updateTagList() {
        TifariAPI.getAllTags().then(tags => { 
            tags.sort((a, b) => a.times_used < b.times_used);
            this.mutateState(mut => 
                mut.setTags(tags)
                   .orderTags()
            )
        });
    }

    doTagsMatchSearch(tags) {
        // searchTagNames and tags should intersect.

        const intersection = this.state.searchTagNames.filter(
            tagName => tags.findIndex(tag => tag.name === tagName) !== -1);

        ldebug("Intersecting tags:");
        ldebug(tags);

        return intersection.length > 0;
    }

    // callback that's called when we remove a tag from an image
    foreignOnEditorRemoveTagFromSelected(tag) {

        let imageIds = this.state.selectedImages.map(img => img.id);

        TifariAPI.removeTags([tag.id], imageIds);

        this.mutateState(mut => {

            mut.getOldState().selectedImages
                .forEach(image => {

                    mut.removeTagFromImage(image, tag);

                    if(mut.getOldState().isInToTagList && 0 >= image.tags.length) {
                        mut.addImageToList(image);
                    } else if(!this.doTagsMatchSearch(image.tags)) {
                        mut.removeImageFromList(image);
                    }

                });
        });

        this.updateTagList();
        this.updateToBeTaggedListSize();
    }
    
    // callback that's called whenever we add a tag to an image
    foreignOnEditorAddTagToSelected(tagNames) { 

        let imageIds = this.state.selectedImages.map(img => img.id);

        TifariAPI.addTags(tagNames, imageIds)
            .then(tags => this.mutateState(mut => {

                // add each tag to each image
                tags.forEach(
                    tag => mut.getOldState().selectedImages.forEach(
                        img => mut.addTagToImage(img, tag))
                );

                // if we're in the to tag list, remove all the selected images
                // since we just added tags to those images.
                if(mut.getOldState().isInToTagList) {
                    mut.getOldState().selectedImages
                        .forEach(img => mut.removeImageFromList(img));
                } else {
                    mut.getOldState().selectedImages.forEach(img =>{
                        if(this.doTagsMatchSearch(img.tags)) {
                            mut.addImageToList(img);
                        }
                    });
                }
            })
        );

        this.updateTagList();
        this.updateToBeTaggedListSize();
    }

    foreignToggleTagListDisplay() {
        this.mutateState(mut => mut.setTagListDisplayState(!mut.getOldState().displayTagList));
    }

    foreignViewToBeTaggedList() {
        TifariAPI.getToBeTaggedList()
            .then(images =>
                this.mutateState(mut => 
                    mut.setImageList(images)
                        .setIsInToBeTaggedList(true))
            );
    }

    foreignAddTagToSearch(tag) {

        if(this.state.searchTagNames.findIndex(t => t === tag.name) !== -1)
            return;

        let search = this.refSearchBar.current;
        search.value = search.value.trim();

        if(0 >= search.value.length) {
            search.value = search.value.concat(tag.name);
        } else {
            search.value = search.value.concat(" ", tag.name);
        }

        this.doImageSearch(search.value);
    }

    removeImageFromSelected(image) {
        this.mutateState(mut => mut.removeSelectedImage(image));
    }

    isImageSelected(image) {
        return -1 !== this.state.selectedImages.findIndex(i => i.id === image.id);
    }

    render() {

        ldebug("Rendering");

        const imageList = this.state.queriedImages.map(img => {
            
            let isSelected = this.isImageSelected(img);
            return(
            <Grid item xs={6}>

            <Card square={true} elevation={5} className="imageField">
                
                <img style={{opacity: isSelected ? 0.5 : 1}}
                    src={TifariAPI.getImageUrl(img)}
                    title={img.path}
                />

                { isSelected && 
                <Icon 
                    className="checkmark" 
                    style={{fontSize: 48}}
                    >
                    done_outline
                </Icon>
                }
                
                <div className="showWhenHovering">

                    { !isSelected &&
                    <Button className="buttonField" onClick={() => this.onSelectImage(img)}>
                        <span className="showWhenHovering--on">
                            Select
                        </span>
                    </Button>
                    }

                    { isSelected &&
                    <Button className="buttonField" onClick={() => this.removeImageFromSelected(img)}>
                        <span className="showWhenHovering--on">
                           Remove 
                        </span>
                    </Button>
                    }
                </div>

            </Card>
            </Grid>
            );
        });

        return (
            <React.Fragment>
            <CssBaseline/>

                {this.state.selectedImages.length > 0 &&
                    <ImageEditor 
                        images = {this.state.selectedImages}
                        onAddTag = {this.foreignOnEditorAddTagToSelected}
                        onRemoveTag = {this.foreignOnEditorRemoveTagFromSelected}
                        callbackRemoveImageFromSelected = {this.foreignRemoveImageFromSelected}
                    />
                }

                {this.state.displayTagList &&
                    <TagList 
                        tags = {this.state.tags}
                        callbackSetOrdering = {this.foreignSetTagListOrdering}
                        callbackAddTag = {this.foreignAddTagToSearch}
                    />
                }
                
                <Paper className="topBar">
                <div className="searchField">
                    <TextField
                        fullWidth = {true}
                        autoFocus = {true}
                        helperText = "Tags"
                        type = "text"
                        ref = {this.refSearchBar}
                        onChange = {ev => this.doImageSearch(ev.target.value.trim())}
                    />
                    <Button onClick={this.foreignViewToBeTaggedList}>
                        To-Tag List({this.state.tagQueueSize})
                    </Button>
                    <Button 
                        onClick={() => TifariAPI.reloadRoot().then(
                            () => this.mutateState(mut => mut.showSnackbar("Reloaded images")))}
                        >
                        Reload Images
                    </Button>

                    <Button onClick={this.foreignToggleTagListDisplay}>
                        {this.state.displayTagList ? "Hide" : "Show"} tag list
                    </Button>

                </div>
                </Paper>
    

                <div className="imageList">
                    <Grid container spacing={16}>
                        {imageList}
                    </Grid>
                </div>

                <Snackbar
                    open={this.state.showSnackbar}
                    anchorOrigin={{ vertical: "bottom", horizontal: "left" }}
                    onClose={() => this.mutateState(mut => mut.hideSnackbar())}
                    ContentProps={{
                        'aria-describedby': 'message-id',
                    }}
                    message={<span id="message-id">{this.state.snackbarMessage}</span>}
             />
               
            </React.Fragment>
        );
    }
}

export default App;
